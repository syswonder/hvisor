#!/usr/bin/env python3
"""Workspace-local rolling CI queue. No Jenkins plugin or server-side service."""

from __future__ import annotations

import argparse
import fcntl
import json
import os
import re
import signal
import sys
import tempfile
import time
from contextlib import contextmanager
from pathlib import Path
from typing import Any, Iterator

from ci_resources import (
    ResourceError, cpu_busy, cpu_times, load_policy, resource_incident,
    select_with_wait, snapshot, summarize_metrics, utc_now,
)


class StateStore:
    """Serialize short state transitions across Jenkins shell workers."""

    def __init__(self, path: Path):
        self.path = path
        self.lock_path = path.with_suffix(path.suffix + ".lock")

    @contextmanager
    def lock(self, *, write: bool = True) -> Iterator[None]:
        self.path.parent.mkdir(parents=True, exist_ok=True)
        with self.lock_path.open("a+b") as lock:
            fcntl.flock(lock, fcntl.LOCK_EX if write else fcntl.LOCK_SH)
            try:
                yield
            finally:
                fcntl.flock(lock, fcntl.LOCK_UN)

    def _write(self, state: dict[str, Any]) -> None:
        name = None
        try:
            with tempfile.NamedTemporaryFile(
                mode="w", encoding="utf-8", dir=self.path.parent, delete=False,
                prefix=".state-",
            ) as stream:
                name = stream.name
                json.dump(state, stream, indent=2)
                stream.write("\n")
            os.replace(name, self.path)
        finally:
            if name is not None and os.path.exists(name):
                os.unlink(name)

    def create(self, state: dict[str, Any]) -> None:
        with self.lock():
            if self.path.exists():
                raise ResourceError(f"refusing to overwrite existing queue: {self.path}")
            self._write(state)

    @contextmanager
    def transaction(self, *, write: bool = True) -> Iterator[dict[str, Any]]:
        with self.lock(write=write):
            with self.path.open(encoding="utf-8") as stream:
                state = json.load(stream)
            yield state
            if write:
                self._write(state)


def order_bids(bids: list[str], priority: list[str]) -> list[str]:
    if not isinstance(bids, list) or not bids:
        raise ResourceError("matrix BID list must not be empty")
    if any(not isinstance(bid, str) or not re.fullmatch(r"[A-Za-z0-9_-]+/[A-Za-z0-9_-]+", bid)
           for bid in bids):
        raise ResourceError("invalid matrix BID")
    if len(set(bids)) != len(bids):
        raise ResourceError("duplicate matrix BID")
    if not set(priority).issubset(bids):
        raise ResourceError("priority BIDs must exist in the matrix")
    return priority + [bid for bid in bids if bid not in priority]


def initial_state(bids: list[str], policy: dict[str, Any],
                  profile: dict[str, Any], observed: dict[str, Any]) -> dict[str, Any]:
    ordered = order_bids(bids, policy["priority_bids"])
    return {
        "version": 1,
        "profile": profile,
        "bids": bids,
        "pending": ordered,
        "active": {},
        "results": {},
        "peak_active": 0,
        "paused_since": None,
        "stop_reason": None,
        "finalized": False,
        "unfinished_bids": [],
        "started": observed,
        "policy": {key: policy[key] for key in (
            "reserve_mib", "pause_below_mib", "resume_at_mib",
            "wait_timeout_seconds", "poll_seconds",
        )},
    }


def claim_next(state: dict[str, Any], worker: str, available: int, now: float) -> dict[str, Any]:
    """Caller holds the workspace lock; checking memory/claiming one BID is atomic."""
    slots = state["profile"]["slots"]
    if worker not in {f"slot-{index}" for index in range(1, slots + 1)}:
        raise ResourceError(f"unknown worker: {worker}")
    if state["finalized"]:
        return {"status": "done"}
    if worker in state["active"]:
        # Refuse to replay a potentially already-started hardware test.
        raise ResourceError(f"{worker} already owns an unfinished BID")
    if state["stop_reason"]:
        return {"status": "stopped", "reason": state["stop_reason"]}
    if not state["pending"]:
        return {"status": "done"}
    if len(state["active"]) >= slots:
        return {"status": "wait", "reason": "all slots in use"}

    policy = state["policy"]
    if available < policy["pause_below_mib"] and state["paused_since"] is None:
        state["paused_since"] = now
    if state["paused_since"] is not None:
        if available >= policy["resume_at_mib"]:
            state["paused_since"] = None
        elif now - state["paused_since"] >= policy["wait_timeout_seconds"]:
            reason = ("CI_RESOURCE_PRESSURE: memory did not recover to "
                      f"{policy['resume_at_mib']} MiB within the dispatch wait")
            state["stop_reason"] = reason
            return {"status": "stopped", "reason": reason, "available_mib": available}
        else:
            return {"status": "wait", "reason": "memory pressure",
                    "available_mib": available}

    bid = state["pending"].pop(0)
    state["active"][worker] = {"bid": bid, "started": utc_now(), "monotonic": now}
    state["peak_active"] = max(state["peak_active"], len(state["active"]))
    return {"status": "run", "bid": bid, "active": len(state["active"]),
            "available_mib": available}


def complete_bid(state: dict[str, Any], worker: str, bid: str,
                 result: str, now: float) -> None:
    if result not in ("SUCCESS", "FAILURE", "ABORTED"):
        raise ResourceError(f"invalid cell result: {result}")
    previous = state["results"].get(bid)
    if previous is not None:
        if previous.get("worker") == worker and previous["result"] == result:
            return  # Idempotent receipt for a completed durable step.
        raise ResourceError(f"conflicting completion for {bid}")
    assignment = state["active"].get(worker)
    if not assignment or assignment["bid"] != bid:
        raise ResourceError(f"{worker} does not own {bid}")
    del state["active"][worker]
    state["results"][bid] = {
        "result": result,
        "worker": worker,
        "started": assignment["started"],
        "finished": utc_now(),
        "duration_seconds": round(max(0, now - assignment["monotonic"]), 3),
    }


def stop_dispatch(state: dict[str, Any], reason: str) -> None:
    if not state["finalized"] and not state["stop_reason"]:
        state["stop_reason"] = reason


def claim_observed(state: dict[str, Any], worker: str, observed: dict[str, Any]) -> dict[str, Any]:
    # A controller restart can interrupt the sampler before its stop is recorded.
    if incident := resource_incident(state["started"], observed):
        stop_dispatch(state, f"CI_RESOURCE_INCIDENT: {incident}")
    return claim_next(state, worker, observed["effective_available_mib"],
                      observed["monotonic"])


def finalize_state(state: dict[str, Any]) -> dict[str, Any]:
    if not state["finalized"]:
        # Unfinished work can never inherit a SUCCESS from another cell or run.
        for worker, assignment in state["active"].items():
            bid = assignment["bid"]
            state["results"][bid] = {
                "result": "ABORTED", "worker": worker,
                "started": assignment["started"], "finished": utc_now(),
            }
            state["unfinished_bids"].append(bid)
        for bid in state["pending"]:
            state["results"][bid] = {"result": "NOT_BUILT"}
            state["unfinished_bids"].append(bid)
        state["active"].clear()
        state["pending"].clear()
        state["finalized"] = True

    results = {bid: state["results"][bid] for bid in state["bids"]}
    counts = {status: sum(item["result"] == status for item in results.values())
              for status in ("SUCCESS", "FAILURE", "ABORTED", "NOT_BUILT")}
    return {
        "profile": state["profile"],
        "total": len(state["bids"]),
        "counts": counts,
        "peak_active": state["peak_active"],
        "stop_reason": state["stop_reason"],
        "unfinished_bids": state["unfinished_bids"],
        "results": results,
        "success": counts["SUCCESS"] == len(results) and not state["stop_reason"],
    }


def monitor(store: StateStore, output: Path, stop_file: Path) -> None:
    """A foreground Jenkins step; no detached monitor or persistent service."""
    output.parent.mkdir(parents=True, exist_ok=True)
    interrupted = False

    def handle_signal(signum: int, frame: Any) -> None:
        nonlocal interrupted
        interrupted = True

    signal.signal(signal.SIGTERM, handle_signal)
    signal.signal(signal.SIGINT, handle_signal)
    previous_cpu = None
    with store.transaction(write=False) as state:
        first = state["started"]
    with output.open("w", encoding="utf-8") as stream:
        while True:
            observed = snapshot(details=True)
            current_cpu = cpu_times(Path("/proc"))
            observed["cpu_busy_percent"] = cpu_busy(previous_cpu, current_cpu)
            previous_cpu = current_cpu
            with store.transaction(write=False) as state:
                observed["active_bids"] = [item["bid"] for item in state["active"].values()]
                observed["pending_count"] = len(state["pending"])
                interval = state["policy"]["poll_seconds"]
            stream.write(json.dumps(observed) + "\n")
            stream.flush()
            if incident := resource_incident(first, observed):
                with store.transaction() as state:
                    stop_dispatch(state, f"CI_RESOURCE_INCIDENT: {incident}")
            if interrupted or stop_file.exists():
                break
            time.sleep(interval)
    if interrupted and not stop_file.exists():
        raise ResourceError("CI_RESOURCE_MONITOR_INTERRUPTED")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    for name in ("init", "claim", "complete", "stop", "finalize", "monitor", "summarize"):
        command = commands.add_parser(name)
        command.add_argument("--state", type=Path, required=True)
        if name == "init":
            command.add_argument("--policy", type=Path, required=True)
            command.add_argument("--bids", type=Path, required=True)
        if name in ("claim", "complete"):
            command.add_argument("--worker", required=True)
        if name == "complete":
            command.add_argument("--bid", required=True)
            command.add_argument("--result", required=True,
                                 choices=["SUCCESS", "FAILURE", "ABORTED"])
        if name == "stop":
            command.add_argument("--reason", required=True)
        if name == "monitor":
            command.add_argument("--output", type=Path, required=True)
            command.add_argument("--stop-file", type=Path, required=True)
        if name == "summarize":
            command.add_argument("--metrics", type=Path, required=True)
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    store = StateStore(args.state)
    result: dict[str, Any] | None = None
    try:
        if args.command == "init":
            policy = load_policy(args.policy)
            bids = json.loads(args.bids.read_text(encoding="utf-8"))
            order_bids(bids, policy["priority_bids"])
            selected, observed = select_with_wait(policy)
            state = initial_state(bids, policy, selected, observed)
            store.create(state)
            result = {"profile": selected, "policy": state["policy"],
                      "available_mib": observed["effective_available_mib"]}
        elif args.command == "monitor":
            monitor(store, args.output, args.stop_file)
        elif args.command == "summarize":
            with store.transaction(write=False) as state:
                policy = state["policy"]
            result = summarize_metrics(args.metrics, policy["reserve_mib"],
                                       policy["resume_at_mib"], state["started"])
        else:
            with store.transaction() as state:
                if args.command == "claim":
                    result = claim_observed(state, args.worker, snapshot(details=True))
                elif args.command == "complete":
                    complete_bid(state, args.worker, args.bid, args.result, time.monotonic())
                elif args.command == "stop":
                    stop_dispatch(state, args.reason)
                elif args.command == "finalize":
                    result = finalize_state(state)
        if result is not None:
            print(json.dumps(result))
        return 0
    except (ResourceError, OSError, ValueError) as error:
        print(f"CI resource error: {error}", file=sys.stderr, flush=True)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
