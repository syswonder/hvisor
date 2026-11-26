import json
import subprocess
import sys
import tempfile
import time
import unittest
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path
from unittest.mock import patch

from ci_resources import ResourceError, load_policy, select_profile
from ci_scheduler import (
    StateStore, claim_next, claim_observed, complete_bid, finalize_state, initial_state,
    monitor, order_bids, stop_dispatch,
)
from sync_bid_matrix import find_bid_values_block


JENKINS_DIR = Path(__file__).resolve().parents[1]
ROOT = JENKINS_DIR.parent


def matrix_bids():
    lines = (ROOT / "Jenkinsfile").read_text().splitlines(keepends=True)
    first, last, _ = find_bid_values_block(lines)
    return [line.strip().strip(",").strip("'") for line in lines[first + 1:last]]


def make_state():
    policy = load_policy(JENKINS_DIR / "resource_policy.json")
    profile = select_profile(policy, 13000)
    return initial_state(matrix_bids(), policy, profile, {
        "effective_available_mib": 13000, "monotonic": 0,
        "cgroup_events": {"oom_kill": 0},
    })


class QueueTests(unittest.TestCase):
    def test_rolling_slot_refill_and_failure_isolation(self):
        state = make_state()
        a = claim_next(state, "slot-1", 13000, 0)["bid"]
        b = claim_next(state, "slot-2", 13000, 1)["bid"]
        self.assertEqual(a, "aarch64/phytium-pi")
        self.assertEqual(b, "x86_64/qemu_asterinas")
        # An ordinary failure frees one slot while the long second test runs.
        complete_bid(state, "slot-1", a, "FAILURE", 5)
        c = claim_next(state, "slot-1", 13000, 6)["bid"]
        self.assertEqual(c, "aarch64/qemu-gicv3")
        self.assertEqual(state["active"]["slot-2"]["bid"], b)
        self.assertEqual(state["peak_active"], 2)
        complete_bid(state, "slot-1", c, "SUCCESS", 7)
        complete_bid(state, "slot-2", b, "SUCCESS", 8)
        while (item := claim_next(state, "slot-1", 13000, 9))["status"] == "run":
            complete_bid(state, "slot-1", item["bid"], "SUCCESS", 10)
        report = finalize_state(state)
        self.assertEqual(report["total"], 21)
        self.assertEqual(report["counts"], {
            "SUCCESS": 20, "FAILURE": 1, "ABORTED": 0, "NOT_BUILT": 0})
        self.assertFalse(report["success"])
        self.assertEqual(report["unfinished_bids"], [])

    def test_pressure_pause_hysteresis_and_recovery(self):
        state = make_state()
        a = claim_next(state, "slot-1", 13000, 0)["bid"]
        self.assertEqual(claim_next(state, "slot-2", 4095, 5)["status"], "wait")
        self.assertEqual(claim_next(state, "slot-2", 4500, 10)["status"], "wait")
        complete_bid(state, "slot-1", a, "SUCCESS", 20)
        self.assertEqual(claim_next(state, "slot-1", 5119, 21)["status"], "wait")
        self.assertEqual(claim_next(state, "slot-2", 5120, 22)["status"], "run")
        self.assertIsNone(state["paused_since"])

    def test_pressure_timeout_does_not_fake_success(self):
        state = make_state()
        a = claim_next(state, "slot-1", 13000, 0)["bid"]
        self.assertEqual(claim_next(state, "slot-2", 2000, 0)["status"], "wait")
        self.assertEqual(claim_next(state, "slot-2", 2000, 300)["status"], "stopped")
        complete_bid(state, "slot-1", a, "SUCCESS", 310)
        self.assertEqual(claim_next(state, "slot-1", 13000, 311)["status"], "stopped")
        report = finalize_state(state)
        self.assertFalse(report["success"])
        self.assertEqual(report["counts"]["NOT_BUILT"], 20)
        self.assertIn("CI_RESOURCE_PRESSURE", report["stop_reason"])

    def test_abort_finalizes_active_and_pending_without_replay(self):
        state = make_state()
        a = claim_next(state, "slot-1", 13000, 0)["bid"]
        b = claim_next(state, "slot-2", 13000, 0)["bid"]
        with self.assertRaisesRegex(ResourceError, "unfinished BID"):
            claim_next(state, "slot-1", 13000, 1)
        with self.assertRaisesRegex(ResourceError, "unknown worker"):
            claim_next(state, "slot-3", 13000, 1)
        complete_bid(state, "slot-1", a, "SUCCESS", 1)
        complete_bid(state, "slot-1", a, "SUCCESS", 2)  # Idempotent receipt.
        with self.assertRaises(ResourceError):
            complete_bid(state, "slot-1", a, "FAILURE", 3)
        stop_dispatch(state, "monitor failed")
        report = finalize_state(state)
        self.assertEqual(report["results"][b]["result"], "ABORTED")
        self.assertEqual(report["counts"]["NOT_BUILT"], 19)
        self.assertEqual(len(report["unfinished_bids"]), 20)
        self.assertEqual(finalize_state(state), report)

    def test_bid_validation(self):
        with self.assertRaises(ResourceError):
            order_bids(["a/b", "a/b"], [])
        with self.assertRaises(ResourceError):
            order_bids(["a/b"], ["x/y"])
        with self.assertRaises(ResourceError):
            order_bids(["a/../escape"], [])

    def test_claim_detects_incident_even_when_monitor_was_interrupted(self):
        state = make_state()
        claim = claim_observed(state, "slot-1", {
            "effective_available_mib": 13000, "monotonic": 1,
            "cgroup_events": {"oom_kill": 1},
        })
        self.assertEqual(claim["status"], "stopped")
        self.assertEqual(len(state["pending"]), 21)


class StateStoreTests(unittest.TestCase):
    def test_concurrent_processes_cannot_duplicate_bids(self):
        with tempfile.TemporaryDirectory() as tmp:
            store = StateStore(Path(tmp) / "state.json")
            store.create(make_state())
            with self.assertRaises(ResourceError):
                store.create(make_state())

            # Real, independent Python processes contend for the same flock.
            code = """
import json, sys
from pathlib import Path
from ci_scheduler import StateStore, claim_next
store = StateStore(Path(sys.argv[1]))
with store.transaction() as state:
    result = claim_next(state, sys.argv[2], 13000, 0)
print(json.dumps(result))
"""
            processes = [
                subprocess.Popen([sys.executable, "-c", code, str(store.path), worker],
                                 cwd=JENKINS_DIR, stdout=subprocess.PIPE, text=True)
                for worker in ("slot-1", "slot-2")
            ]
            claims = [json.loads(proc.communicate(timeout=10)[0]) for proc in processes]
            self.assertTrue(all(proc.returncode == 0 for proc in processes))
            self.assertEqual(len({claim["bid"] for claim in claims}), 2)
            with store.transaction(write=False) as state:
                self.assertEqual(len(state["active"]), 2)
                self.assertEqual(state["peak_active"], 2)

    def test_all_21_execute_once_with_concurrent_workers(self):
        with tempfile.TemporaryDirectory() as tmp:
            store = StateStore(Path(tmp) / "state.json")
            store.create(make_state())

            def drain(worker):
                seen = []
                while True:
                    with store.transaction() as state:
                        item = claim_next(state, worker, 13000, time.monotonic())
                    if item["status"] == "done":
                        return seen
                    self.assertEqual(item["status"], "run")
                    seen.append(item["bid"])
                    time.sleep(0.005)
                    with store.transaction() as state:
                        complete_bid(state, worker, item["bid"], "SUCCESS", time.monotonic())

            with ThreadPoolExecutor(max_workers=2) as pool:
                first, second = pool.map(drain, ("slot-1", "slot-2"))
            self.assertEqual(len(first + second), 21)
            self.assertEqual(len(set(first + second)), 21)
            with store.transaction() as state:
                report = finalize_state(state)
            self.assertTrue(report["success"])
            self.assertEqual(report["peak_active"], 2)
            self.assertEqual(report["counts"]["SUCCESS"], 21)

    def test_monitor_records_and_stops_dispatch_on_incident(self):
        with tempfile.TemporaryDirectory() as tmp:
            directory = Path(tmp)
            store = StateStore(directory / "state.json")
            store.create(make_state())
            stop_file = directory / "monitor.stop"
            stop_file.touch()
            output = directory / "metrics.jsonl"
            with patch("ci_scheduler.snapshot", return_value={
                "effective_available_mib": 2000, "monotonic": 1,
                "cgroup_events": {"oom_kill": 1},
            }), patch("ci_scheduler.cpu_times", return_value=None), \
                 patch("ci_scheduler.signal.signal"):
                monitor(store, output, stop_file)
            self.assertEqual(len(output.read_text().splitlines()), 1)
            with store.transaction(write=False) as state:
                self.assertIn("CI_RESOURCE_INCIDENT", state["stop_reason"])

    def test_unexpected_monitor_interrupt_is_not_a_normal_stop(self):
        with tempfile.TemporaryDirectory() as tmp:
            directory = Path(tmp)
            store = StateStore(directory / "state.json")
            store.create(make_state())
            handlers = []

            def register(signum, handler):
                handlers.append(handler)

            def observed():
                handlers[0](None, None)
                return {"effective_available_mib": 13000, "monotonic": 1,
                        "cgroup_events": {"oom_kill": 0}}

            with patch("ci_scheduler.snapshot", side_effect=lambda **kwargs: observed()), \
                 patch("ci_scheduler.cpu_times", return_value=None), \
                 patch("ci_scheduler.signal.signal", side_effect=register):
                with self.assertRaisesRegex(ResourceError, "MONITOR_INTERRUPTED"):
                    monitor(store, directory / "metrics.jsonl", directory / "monitor.stop")

    def test_cli_lifecycle_with_foreground_monitor(self):
        # Exercise the exact command-line contract used by the Jenkins workers.
        # The tiny thresholds are a temporary test policy, not the CI policy.
        with tempfile.TemporaryDirectory() as tmp:
            directory = Path(tmp)
            policy = json.loads((JENKINS_DIR / "resource_policy.json").read_text())
            policy.update(reserve_mib=1, pause_below_mib=2, resume_at_mib=3,
                          poll_seconds=1, wait_timeout_seconds=2)
            policy["pilot_profile"]["min_start_available_mib"] = 3
            policy_path = directory / "policy.json"
            bids_path = directory / "bids.json"
            state_path = directory / "state.json"
            stop_file = directory / "monitor.stop"
            metrics = directory / "metrics.jsonl"
            policy_path.write_text(json.dumps(policy))
            bids_path.write_text(json.dumps(matrix_bids()))

            def command(name, *args):
                return [sys.executable, str(JENKINS_DIR / "ci_scheduler.py"), name,
                        "--state", str(state_path), *map(str, args)]

            def invoke(name, *args):
                result = subprocess.run(command(name, *args), capture_output=True,
                                        text=True, check=True, timeout=10)
                return json.loads(result.stdout) if result.stdout else None

            setup = invoke("init", "--policy", policy_path, "--bids", bids_path)
            self.assertEqual(setup["profile"]["slots"], 2)
            sampler = subprocess.Popen(command("monitor", "--output", metrics,
                                               "--stop-file", stop_file),
                                       stdout=subprocess.PIPE, stderr=subprocess.PIPE)
            try:
                def drain(worker):
                    seen = []
                    while True:
                        claim = invoke("claim", "--worker", worker)
                        if claim["status"] == "done":
                            return seen
                        self.assertEqual(claim["status"], "run")
                        bid = claim["bid"]
                        seen.append(bid)
                        # No Hvisor build or QEMU/board command is executed.
                        time.sleep(0.01)
                        invoke("complete", "--worker", worker, "--bid", bid,
                               "--result", "SUCCESS")

                with ThreadPoolExecutor(max_workers=2) as pool:
                    first, second = pool.map(drain, ("slot-1", "slot-2"))
                self.assertEqual(len(set(first + second)), 21)
            finally:
                stop_file.touch()
                try:
                    _, stderr = sampler.communicate(timeout=10)
                except subprocess.TimeoutExpired:
                    sampler.kill()
                    sampler.communicate()
                    raise
            self.assertEqual(sampler.returncode, 0, stderr.decode())
            result = invoke("finalize")
            summary = invoke("summarize", "--metrics", metrics)
            self.assertTrue(result["success"])
            self.assertEqual(result["peak_active"], 2)
            self.assertEqual(result["counts"]["SUCCESS"], 21)
            self.assertGreaterEqual(summary["samples"], 1)
            self.assertEqual(summary["malformed_samples"], 0)


if __name__ == "__main__":
    unittest.main()
