import copy
import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from ci_resources import (
    MIB,
    ResourceError,
    cpu_busy,
    load_policy,
    resource_incident,
    select_profile,
    select_with_wait,
    snapshot,
    summarize_metrics,
    validate_policy,
)


JENKINS_DIR = Path(__file__).resolve().parents[1]


class ProfileTests(unittest.TestCase):
    def setUp(self):
        self.raw = json.loads((JENKINS_DIR / "resource_policy.json").read_text())

    def test_initial_profile_is_explicitly_unvalidated(self):
        policy = load_policy(JENKINS_DIR / "resource_policy.json")
        self.assertIsNone(select_profile(policy, 11263))
        profile = select_profile(policy, 11264)
        self.assertEqual((profile["slots"], profile["cargo_jobs"]), (2, 2))
        self.assertFalse(profile["validated"])

    def test_select_fastest_validated_profile_or_lower_profile(self):
        self.raw["validated_profiles"] = [
            {
                "name": "slow",
                "slots": 1,
                "cargo_jobs": 1,
                "min_start_available_mib": 6500,
                "duration_seconds": 900,
                "successful_runs": 2,
            },
            {
                "name": "fast",
                "slots": 3,
                "cargo_jobs": 2,
                "min_start_available_mib": 11000,
                "duration_seconds": 400,
                "successful_runs": 2,
            },
        ]
        policy = validate_policy(self.raw)
        self.assertEqual(select_profile(policy, 12000)["name"], "fast")
        self.assertEqual(select_profile(policy, 8000)["name"], "slow")
        # No unvalidated fallback even if the pilot configuration could fit.
        self.assertIsNone(select_profile(policy, 6000))

    def test_no_pilot_after_profiles_are_validated(self):
        self.raw["validated_profiles"] = [
            {
                "name": "high",
                "slots": 4,
                "cargo_jobs": 2,
                "min_start_available_mib": 14000,
                "duration_seconds": 500,
                "successful_runs": 2,
            }
        ]
        self.assertIsNone(select_profile(validate_policy(self.raw), 12000))

    def test_reject_invalid_or_unbounded_profiles(self):
        for changes in (
            {"slots": 21},
            {"cargo_jobs": 24},
            {"slots": 0},
            {"min_start_available_mib": 1024},
            {"cargo_jobs": True},
        ):
            with self.subTest(changes=changes):
                raw = copy.deepcopy(self.raw)
                raw["pilot_profile"].update(changes)
                with self.assertRaises(ResourceError):
                    validate_policy(raw)
        self.raw["validated_profiles"] = [
            {
                "name": "unproven",
                "slots": 3,
                "cargo_jobs": 2,
                "min_start_available_mib": 12000,
                "duration_seconds": 400,
                "successful_runs": 1,
            }
        ]
        with self.assertRaises(ResourceError):
            validate_policy(self.raw)

    def test_preflight_wait_is_bounded(self):
        policy = validate_policy(self.raw)
        with (
            patch(
                "ci_resources.snapshot", return_value={"effective_available_mib": 1000}
            ),
            patch("ci_resources.time.monotonic", side_effect=[0, 301]),
            patch("ci_resources.time.sleep") as sleep,
        ):
            with self.assertRaisesRegex(ResourceError, "CI_RESOURCE_PRESSURE"):
                select_with_wait(policy)
            sleep.assert_not_called()


class ObservationTests(unittest.TestCase):
    def test_host_and_ancestor_cgroup_headroom(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            proc = root / "proc"
            cgroup = root / "cgroup"
            leaf = cgroup / "system.slice" / "jenkins.service"
            (proc / "self").mkdir(parents=True)
            leaf.mkdir(parents=True)
            (proc / "meminfo").write_text(
                "MemTotal: 16000000 kB\nMemAvailable: 12288000 kB\n"
                "SwapTotal: 4194304 kB\nSwapFree: 3145728 kB\n"
            )
            (proc / "self/cgroup").write_text("0::/system.slice/jenkins.service\n")
            (leaf / "memory.current").write_text(str(3000 * MIB))
            (leaf / "memory.max").write_text("max")
            (leaf / "memory.events").write_text("oom 1\noom_kill 0\n")
            (leaf.parent / "memory.current").write_text(str(5000 * MIB))
            (leaf.parent / "memory.max").write_text(str(9000 * MIB))
            observed = snapshot(proc, cgroup)
            self.assertEqual(observed["mem_available_mib"], 12000)
            self.assertEqual(observed["effective_available_mib"], 4000)
            self.assertEqual(observed["cgroup_current_mib"], 3000)
            self.assertEqual(observed["swap_used_mib"], 1024)
            self.assertEqual(observed["cgroup_events"]["oom_kill"], 0)

    def test_fail_closed_without_memavailable(self):
        with tempfile.TemporaryDirectory() as tmp:
            proc = Path(tmp)
            (proc / "meminfo").write_text("MemTotal: 10000 kB\n")
            with self.assertRaises(ResourceError):
                snapshot(proc, proc / "cgroup")

    def test_cpu_and_incidents(self):
        self.assertEqual(cpu_busy((100, 30), (200, 70)), 60.0)
        self.assertIsNone(cpu_busy(None, (200, 70)))
        baseline = {"cgroup_events": {"oom_kill": 1}, "jenkins_main_pid": 123}
        self.assertIsNone(resource_incident(baseline, baseline))
        self.assertIn(
            "oom_kill", resource_incident(baseline, {"cgroup_events": {"oom_kill": 2}})
        )
        self.assertIn(
            "jenkins_main_pid", resource_incident(baseline, {"jenkins_main_pid": 456})
        )
        self.assertIsNone(resource_incident(baseline, {}))

    def test_summary_uses_preflight_and_distinguishes_missing_data(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "metrics.jsonl"
            baseline = {
                "effective_available_mib": 12000,
                "monotonic": 0,
                "swap_used_mib": 0,
                "jenkins_main_pid": 100,
                "jenkins_restarts": 3,
                "cgroup_events": {"oom_kill": 0},
            }
            rows = [
                {
                    "effective_available_mib": 8000,
                    "monotonic": 5,
                    "swap_used_mib": 100,
                    "cpu_busy_percent": 75,
                    "jenkins_main_pid": 100,
                    "jenkins_restarts": 3,
                    "cgroup_events": {"oom_kill": 0},
                },
                {
                    "effective_available_mib": 6000,
                    "monotonic": 10,
                    "swap_used_mib": 200,
                    "cpu_busy_percent": 50,
                    "jenkins_main_pid": 100,
                    "jenkins_restarts": 3,
                    "cgroup_events": {"oom_kill": 0},
                },
            ]
            path.write_text("".join(json.dumps(row) + "\n" for row in rows))
            report = summarize_metrics(path, 3072, 5120, baseline)
            self.assertEqual(report["samples"], 2)
            self.assertEqual(report["observed_drop_mib"], 6000)
            self.assertEqual(report["suggested_min_start_available_mib"], 10572)
            self.assertEqual(report["max_swap_increase_mib"], 200)
            self.assertEqual(report["average_cpu_busy_percent"], 62.5)
            self.assertTrue(report["meets_reserve"])
            self.assertIsNone(report["resource_incident"])
            with path.open("a") as stream:
                stream.write("{truncated\n")
            self.assertEqual(
                summarize_metrics(path, 3072, 5120)["malformed_samples"], 1
            )
            self.assertEqual(
                summarize_metrics(path.with_name("missing"), 3072, 5120)["samples"], 0
            )

    def test_summary_preserves_observed_oom_even_if_later_sample_lacks_counter(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "metrics.jsonl"
            base = {
                "effective_available_mib": 10000,
                "monotonic": 0,
                "cgroup_events": {"oom_kill": 0},
            }
            rows = [
                {
                    "effective_available_mib": 2000,
                    "monotonic": 5,
                    "cgroup_events": {"oom_kill": 1},
                },
                {"effective_available_mib": 10000, "monotonic": 10},
            ]
            path.write_text("\n".join(json.dumps(row) for row in rows) + "\n")
            report = summarize_metrics(path, 3072, 5120, base)
            self.assertFalse(report["meets_reserve"])
            self.assertIn("oom_kill", report["resource_incident"])


if __name__ == "__main__":
    unittest.main()
