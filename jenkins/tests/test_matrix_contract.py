import re
import tempfile
import unittest
from pathlib import Path

from ci_resources import load_policy
from ci_scheduler import order_bids
from sync_bid_matrix import find_bid_values_block, sync_jenkinsfile


ROOT = Path(__file__).resolve().parents[2]
# PR #452 d02efb6 baseline. Deliberate coverage changes must update this contract.
EXPECTED_BIDS = [
    "aarch64/imx8mp", "aarch64/jeston-orin", "aarch64/ok6254-c",
    "aarch64/phytium-pi", "aarch64/qemu-gicv2", "aarch64/qemu-gicv3",
    "aarch64/rk3568", "aarch64/rk3588", "aarch64/sysoul_x3300",
    "aarch64/zcu102", "loongarch64/ls3a5000", "loongarch64/ls3a6000",
    "riscv64/hifive-premier-p550", "riscv64/megrez", "riscv64/qemu-aia",
    "riscv64/qemu-plic", "riscv64/ur-dp1000", "x86_64/ecx-2300f-peg",
    "x86_64/nuc14mnk", "x86_64/qemu", "x86_64/qemu_asterinas",
]
EXPECTED_TESTS = {
    "aarch64/phytium-pi", "aarch64/qemu-gicv3", "riscv64/qemu-plic",
    "x86_64/qemu", "x86_64/qemu_asterinas",
}


class MatrixContractTests(unittest.TestCase):
    def test_all_original_checks_and_functional_test_bids(self):
        text = (ROOT / "Jenkinsfile").read_text()
        lines = text.splitlines(keepends=True)
        first, last, _ = find_bid_values_block(lines)
        bids = [line.strip().strip(",").strip("'") for line in lines[first + 1:last]]
        self.assertEqual(bids, EXPECTED_BIDS)
        ci_config = (ROOT / "jenkins/ci.yaml").read_text()
        test_bids = set(re.findall(r"^  - bid: (\S+)\s*$", ci_config, re.M))
        self.assertEqual(test_bids, EXPECTED_TESTS)
        self.assertTrue(test_bids.issubset(bids))
        policy = load_policy(ROOT / "jenkins/resource_policy.json")
        ordered = order_bids(bids, policy["priority_bids"])
        self.assertEqual(set(ordered), set(EXPECTED_BIDS))
        self.assertEqual(set(ordered[:5]), EXPECTED_TESTS)
        self.assertIn("publishGithubCheckInProgress('linter')", text)
        self.assertIn("publishGithubCheckInProgress('license-checker')", text)
        self.assertIn("Build in-tree hvisor-tool", text)

    def test_matrix_sync_supports_old_and_new_syntax(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "Jenkinsfile"
            for fixture in (
                "def matrixBids() {\n    return [\n        'arch/old',\n    ]\n}\n",
                "axis {\n    name 'BID'\n    values(\n        'arch/old',\n    )\n}\n",
            ):
                with self.subTest(fixture=fixture):
                    path.write_text(fixture)
                    self.assertTrue(sync_jenkinsfile(path, ["arch/new"], write=False))
                    self.assertEqual(path.read_text(), fixture)
                    self.assertTrue(sync_jenkinsfile(path, ["arch/new"], write=True))
                    self.assertFalse(sync_jenkinsfile(path, ["arch/new"], write=False))
                    self.assertIn("'arch/new'", path.read_text())


if __name__ == "__main__":
    unittest.main()
