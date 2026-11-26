#!/usr/bin/env python3
"""Read-only Linux resource observations and reviewed CI profile selection."""

from __future__ import annotations

import json
import math
import re
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path
from typing import Any


MIB = 1024 * 1024


class ResourceError(RuntimeError):
    """An unsafe or unavailable CI resource configuration."""


def utc_now() -> str:
    return datetime.now(timezone.utc).isoformat(timespec="seconds")


def positive_int(value: Any, name: str) -> int:
    if type(value) is not int or value <= 0:
        raise ResourceError(f"{name} must be a positive integer")
    return value


def validate_policy(raw: dict[str, Any]) -> dict[str, Any]:
    policy = dict(raw)
    for key in ("reserve_mib", "pause_below_mib", "resume_at_mib",
                "wait_timeout_seconds", "poll_seconds"):
        policy[key] = positive_int(policy.get(key), key)
    if not policy["reserve_mib"] < policy["pause_below_mib"] < policy["resume_at_mib"]:
        raise ResourceError("require reserve_mib < pause_below_mib < resume_at_mib")
    if policy["poll_seconds"] > policy["wait_timeout_seconds"]:
        raise ResourceError("poll_seconds must not exceed wait_timeout_seconds")
    priority = policy.get("priority_bids", [])
    if not isinstance(priority, list) or any(not isinstance(bid, str) for bid in priority):
        raise ResourceError("priority_bids must be a list of BIDs")
    if len(set(priority)) != len(priority):
        raise ResourceError("duplicate priority BID")
    policy["priority_bids"] = priority

    def profile(item: Any, validated: bool) -> dict[str, Any]:
        if not isinstance(item, dict):
            raise ResourceError("profile must be an object")
        name = item.get("name")
        if not isinstance(name, str) or not re.fullmatch(r"[A-Za-z0-9_.-]+", name):
            raise ResourceError("profile name must be a non-empty identifier")
        slots = positive_int(item.get("slots"), f"{name}.slots")
        jobs = positive_int(item.get("cargo_jobs"), f"{name}.cargo_jobs")
        minimum = positive_int(item.get("min_start_available_mib"),
                               f"{name}.min_start_available_mib")
        # This rollout calibrates at most four cells / three Cargo jobs.
        if slots > 4 or jobs > 3:
            raise ResourceError(f"{name}: exceeds the 4-cell / 3-job calibration range")
        if minimum < policy["resume_at_mib"]:
            raise ResourceError(f"{name}: startup memory must be >= resume_at_mib")
        result = {
            "name": name, "slots": slots, "cargo_jobs": jobs,
            "min_start_available_mib": minimum, "validated": validated,
        }
        if validated:
            duration = item.get("duration_seconds")
            runs = positive_int(item.get("successful_runs"), f"{name}.successful_runs")
            if (isinstance(duration, bool) or not isinstance(duration, (int, float))
                    or not math.isfinite(duration) or duration <= 0 or runs < 2):
                raise ResourceError(f"{name}: need measured duration and >= 2 successful runs")
            result.update(duration_seconds=duration, successful_runs=runs)
        return result

    policy["pilot_profile"] = profile(policy.get("pilot_profile"), False)
    validated = policy.get("validated_profiles", [])
    if not isinstance(validated, list):
        raise ResourceError("validated_profiles must be a list")
    policy["validated_profiles"] = [profile(item, True) for item in validated]
    names = [item["name"] for item in policy["validated_profiles"]]
    if len(set(names)) != len(names):
        raise ResourceError("duplicate validated profile name")
    return policy


def load_policy(path: Path) -> dict[str, Any]:
    with path.open(encoding="utf-8") as stream:
        raw = json.load(stream)
    if not isinstance(raw, dict):
        raise ResourceError("resource policy must be an object")
    return validate_policy(raw)


def select_profile(policy: dict[str, Any], available_mib: int) -> dict[str, Any] | None:
    """Use only reviewed profiles once available; never auto-promote a pilot."""
    candidates = policy["validated_profiles"] or [policy["pilot_profile"]]
    suitable = [item for item in candidates
                if available_mib >= item["min_start_available_mib"]]
    if not suitable:
        return None
    return min(suitable, key=lambda item: (
        item.get("duration_seconds", 0), item["slots"], item["cargo_jobs"]
    )).copy()


def read_meminfo(path: Path) -> dict[str, int]:
    values: dict[str, int] = {}
    for line in path.read_text(encoding="utf-8").splitlines():
        key, sep, rest = line.partition(":")
        fields = rest.split()
        if sep and len(fields) >= 2 and fields[1] == "kB":
            values[key] = int(fields[0])
    if "MemAvailable" not in values or "MemTotal" not in values:
        raise ResourceError(f"MemAvailable/MemTotal missing from {path}")
    return values


def read_int(path: Path) -> int | None:
    try:
        return int(path.read_text(encoding="utf-8").strip())
    except (OSError, ValueError):
        return None


def read_counters(path: Path) -> dict[str, int]:
    try:
        return {key: int(value) for key, value in
                (line.split() for line in path.read_text(encoding="utf-8").splitlines())}
    except (OSError, ValueError):
        return {}


def cgroup_memory(proc_root: Path, cgroup_root: Path) -> dict[str, Any]:
    """Respect finite cgroup-v2 ancestor limits in addition to host MemAvailable."""
    try:
        groups = (proc_root / "self/cgroup").read_text(encoding="utf-8").splitlines()
        relative = next(line.split(":", 2)[2] for line in groups if line.startswith("0::"))
        directory = cgroup_root / relative.lstrip("/")
    except (OSError, StopIteration):
        return {}
    if not directory.is_dir():
        return {}

    current = read_int(directory / "memory.current")
    limit = read_int(directory / "memory.max")
    headrooms = []
    for ancestor in (directory, *directory.parents):
        if ancestor != cgroup_root and cgroup_root not in ancestor.parents:
            break
        maximum = read_int(ancestor / "memory.max")
        used = read_int(ancestor / "memory.current")
        if maximum is not None and used is not None:
            headrooms.append(max(0, maximum - used) // MIB)
        if ancestor == cgroup_root:
            break
    return {
        "cgroup_current_mib": current // MIB if current is not None else None,
        "cgroup_limit_mib": limit // MIB if limit is not None else None,
        "cgroup_headroom_mib": min(headrooms) if headrooms else None,
        "cgroup_events": read_counters(directory / "memory.events"),
    }


def cpu_times(proc_root: Path) -> tuple[int, int] | None:
    try:
        fields = (proc_root / "stat").read_text(encoding="utf-8").splitlines()[0].split()
        if fields[0] != "cpu" or len(fields) < 5:
            return None
        ticks = [int(value) for value in fields[1:9]]
        idle = ticks[3] + (ticks[4] if len(ticks) > 4 else 0)
        return sum(ticks), idle
    except (OSError, IndexError, ValueError):
        return None


def cpu_busy(previous: tuple[int, int] | None, current: tuple[int, int] | None) -> float | None:
    if previous is None or current is None:
        return None
    total, idle = current[0] - previous[0], current[1] - previous[1]
    if total <= 0 or not 0 <= idle <= total:
        return None
    return round(100 * (total - idle) / total, 2)


def memory_pressure(proc_root: Path) -> dict[str, float]:
    try:
        return {
            fields[0]: float(fields[1].split("=", 1)[1])
            for fields in (line.split() for line in
                           (proc_root / "pressure/memory").read_text(encoding="utf-8").splitlines())
            if len(fields) >= 2 and fields[1].startswith("avg10=")
        }
    except (OSError, ValueError):
        return {}


def jenkins_service_status() -> dict[str, int]:
    """Best-effort identity/restart observations; no elevated permissions."""
    try:
        result = subprocess.run(
            ["systemctl", "show", "jenkins", "-p", "MainPID", "-p", "NRestarts"],
            capture_output=True, text=True, timeout=2, check=True,
        )
        values = dict(line.split("=", 1) for line in result.stdout.splitlines() if "=" in line)
        return {
            "jenkins_main_pid": int(values["MainPID"]),
            "jenkins_restarts": int(values["NRestarts"]),
        }
    except (OSError, subprocess.SubprocessError, ValueError, KeyError):
        return {}


def snapshot(proc_root: Path = Path("/proc"), cgroup_root: Path = Path("/sys/fs/cgroup"),
             *, details: bool = False) -> dict[str, Any]:
    mem = read_meminfo(proc_root / "meminfo")
    group = cgroup_memory(proc_root, cgroup_root)
    available = mem["MemAvailable"] // 1024
    effective = min(available, group["cgroup_headroom_mib"]) if (
        group.get("cgroup_headroom_mib") is not None
    ) else available
    result = {
        "timestamp": utc_now(),
        "monotonic": time.monotonic(),
        "mem_total_mib": mem["MemTotal"] // 1024,
        "mem_available_mib": available,
        "effective_available_mib": effective,
        "swap_used_mib": ((mem["SwapTotal"] - mem["SwapFree"]) // 1024
                          if "SwapTotal" in mem and "SwapFree" in mem else None),
        **group,
    }
    if details:
        result.update(
            memory_psi_avg10=memory_pressure(proc_root),
            **jenkins_service_status(),
        )
    return result


def select_with_wait(policy: dict[str, Any]) -> tuple[dict[str, Any], dict[str, Any]]:
    deadline = time.monotonic() + policy["wait_timeout_seconds"]
    while True:
        # Baseline before workers start, including the current service identity.
        observed = snapshot(details=True)
        selected = select_profile(policy, observed["effective_available_mib"])
        if selected is not None:
            return selected, observed
        remaining = deadline - time.monotonic()
        if remaining <= 0:
            raise ResourceError(
                "CI_RESOURCE_PRESSURE: no eligible profile; effective available memory "
                f"{observed['effective_available_mib']} MiB after waiting"
            )
        print(f"CI_RESOURCE_WAIT: {observed['effective_available_mib']} MiB available; "
              "no eligible profile", file=sys.stderr, flush=True)
        time.sleep(min(remaining, policy["poll_seconds"]))


def resource_incident(first: dict[str, Any], last: dict[str, Any]) -> str | None:
    for key in ("oom_kill", "oom_group_kill"):
        before = first.get("cgroup_events", {}).get(key)
        after = last.get("cgroup_events", {}).get(key)
        if before is not None and after is not None and after > before:
            return f"Jenkins cgroup {key} increased ({before} -> {after})"
    for key in ("jenkins_main_pid", "jenkins_restarts"):
        before, after = first.get(key), last.get(key)
        if before is not None and after is not None and before != after:
            return f"{key} changed ({before} -> {after})"
    return None


def summarize_metrics(path: Path, reserve_mib: int, resume_at_mib: int,
                      baseline: dict[str, Any] | None = None) -> dict[str, Any]:
    """Summarize observations, not a claim that no host-wide OOM occurred."""
    rows = []
    malformed = 0
    if path.is_file():
        with path.open(encoding="utf-8") as stream:
            for line in stream:
                try:
                    row = json.loads(line)
                    if not isinstance(row, dict) or "effective_available_mib" not in row:
                        raise ValueError("missing memory sample")
                    rows.append(row)
                except (ValueError, TypeError):
                    malformed += 1
    report: dict[str, Any] = {"samples": len(rows), "malformed_samples": malformed}
    if not rows:
        return report
    first, last = baseline or rows[0], rows[-1]
    observations = [first, *rows]
    low = min(row["effective_available_mib"] for row in observations)
    drop = max(0, first["effective_available_mib"] - low)
    swap = [row["swap_used_mib"] for row in observations if row.get("swap_used_mib") is not None]
    cpu = [row["cpu_busy_percent"] for row in rows if row.get("cpu_busy_percent") is not None]
    current = [row["cgroup_current_mib"] for row in observations
               if row.get("cgroup_current_mib") is not None]
    incidents = [incident for row in rows if (incident := resource_incident(first, row))]
    report.update(
        start_available_mib=first["effective_available_mib"],
        min_available_mib=low,
        observed_drop_mib=drop,
        meets_reserve=low >= reserve_mib,
        suggested_min_start_available_mib=max(
            resume_at_mib, reserve_mib + math.ceil(1.25 * drop)),
        max_swap_increase_mib=(max(0, max(swap) - swap[0]) if swap else None),
        max_cgroup_current_mib=max(current) if current else None,
        average_cpu_busy_percent=round(sum(cpu) / len(cpu), 2) if cpu else None,
        observed_seconds=round(max(0, last["monotonic"] - first["monotonic"]), 2),
        resource_incident=incidents[0] if incidents else None,
        first_jenkins_pid=first.get("jenkins_main_pid"),
        last_jenkins_pid=last.get("jenkins_main_pid"),
        first_jenkins_restarts=first.get("jenkins_restarts"),
        last_jenkins_restarts=last.get("jenkins_restarts"),
    )
    return report
