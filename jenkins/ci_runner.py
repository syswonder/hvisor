#!/usr/bin/env python3
from __future__ import annotations

import argparse
import os
import re
import socket
import signal
import subprocess
import time
from pathlib import Path
from typing import Any, Callable

from ci_config import get_bid_entry, load_ci, parse_bid
from terminal import Terminal, TerminalCommandError, TerminalTimeoutError


CaseFunc = Callable[[dict[str, Any], Terminal | None], int]

# Wait for zone1 inner console: shell prompt (#/$) or login:.
ZONE0_READY_PATTERN = r"root@[^\r\n]*[#$]\s|(?:\r?\n)#\s"
ZONE1_INNER_PROMPT_TIMEOUT = 60.0
ZONE1_INNER_LOG_FETCH_TIMEOUT = 30.0


def logs_dir(cfg: dict[str, Any]) -> Path:
    """Per-cell log directory: ``<matrix-cell-workspace>/logs``."""
    path = Path(cfg["workspace"]) / "logs"
    path.mkdir(parents=True, exist_ok=True)
    return path


def wait_qemu_socket(path: str, timeout: float = 30.0) -> None:
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        if Path(path).exists():
            try:
                sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
                sock.settimeout(0.5)
                sock.connect(path)
                sock.close()
                return
            except OSError:
                pass
        time.sleep(0.2)
    raise SystemExit(f"qemu socket not ready: {path}")


def terminate_managed_process(cfg: dict[str, Any]) -> None:
    proc = cfg.get("_managed_proc")
    if proc is None or proc.poll() is not None:
        return
    try:
        os.killpg(proc.pid, signal.SIGTERM)
    except ProcessLookupError:
        return
    try:
        proc.wait(timeout=5.0)
    except subprocess.TimeoutExpired:
        try:
            os.killpg(proc.pid, signal.SIGKILL)
        except ProcessLookupError:
            pass


def parse_pts(output: str) -> list[int]:
    return sorted({int(match) for match in re.findall(r"/dev/pts/(\d+)", output)})


def find_zone1_pts(term: Terminal) -> int:
    """List virtio-console pts devices; retry once on failure."""
    for attempt in range(2):
        _, pts_output = term.run(
            f"zone1_pts_{attempt}",
            "ls -1 /dev/pts/[0-9]*",
            timeout=15.0,
        )
        pts_numbers = parse_pts(pts_output)
        if pts_numbers:
            return pts_numbers[-1]
        if attempt == 0:
            time.sleep(2.0)
    raise TerminalCommandError("failed to find numeric pts from 'ls -1 /dev/pts/[0-9]*'")


def ensure_qemu_terminal(cfg: dict[str, Any], log_path: Path) -> Terminal:
    term = cfg.get("_qemu_term")
    if term is None:
        term = build_terminal(cfg, log_path)
        term.open()
        cfg["_qemu_term"] = term
        cfg["_zone0_log_path"] = log_path
    return term


def close_qemu_terminal(cfg: dict[str, Any]) -> None:
    term = cfg.get("_qemu_term")
    if term is not None:
        term.close()
        cfg["_qemu_term"] = None


def close_board_terminal(cfg: dict[str, Any]) -> None:
    term = cfg.get("_board_term")
    if term is not None:
        term.close()
        cfg["_board_term"] = None


def get_active_terminal(cfg: dict[str, Any]) -> Terminal | None:
    return cfg.get("_qemu_term") or cfg.get("_board_term")


def close_active_terminal(cfg: dict[str, Any]) -> None:
    close_qemu_terminal(cfg)
    close_board_terminal(cfg)


def board_power_script(cfg: dict[str, Any]) -> Path:
    return cfg["workspace"] / "jenkins" / "board_power.sh"


def board_power_cycle(cfg: dict[str, Any]) -> None:
    power_port = str(cfg.get("power_serial", "")).strip()
    if not power_port:
        return
    script = board_power_script(cfg)
    if not script.is_file():
        raise SystemExit(f"board power script not found: {script}")
    subprocess.run(["bash", str(script), "cycle", power_port], check=True, cwd=cfg["workspace"])


def board_wake_console(term: Terminal, *, repeats: int = 3) -> None:
    for _ in range(repeats):
        term.send("")
        time.sleep(0.2)


def board_wait_uboot_prompt(term: Terminal, pattern: str, timeout: float) -> None:
    """Wait for U-Boot prompt, periodically waking an idle console."""
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        remaining = deadline - time.monotonic()
        if term.wait_pattern(pattern, timeout=min(5.0, remaining), from_offset=0):
            return
        board_wake_console(term)
    raise TerminalTimeoutError(f"timed out waiting for U-Boot prompt (pattern={pattern!r})")


def save_inner_serial_log(cfg: dict[str, Any], content: str) -> None:
    if not content:
        return
    path = logs_dir(cfg) / "zone1_inner_serial.log"
    path.write_text(content, encoding="utf-8")


def zone0_start(cfg: dict[str, Any], term: Terminal | None) -> int:
    print("————————————————\ncase: zone0_start\n————————————————\n", flush=True)
    if cfg["mode"] == "qemu":
        # Create log dir before starting QEMU so a permission failure does not
        # leave a running guest that must be SIGTERM'd from finally.
        log_path = logs_dir(cfg) / "zone0_console.log"
        log_path.write_text("", encoding="utf-8")

        cmd = ["make", f"ARCH={cfg['arch']}", f"BOARD={cfg['board']}", "MODE=release", "ci-run"]
        proc = subprocess.Popen(cmd, cwd=cfg["workspace"], start_new_session=True)
        cfg["_managed_proc"] = proc
        cfg["_managed_proc_name"] = "qemu ci-run"
        wait_qemu_socket(cfg["socket_path"], timeout=30.0)

        qemu_term = ensure_qemu_terminal(cfg, log_path)
        uboot_cmd = cfg.get("uboot_cmd", "")
        uboot_ready = cfg.get("uboot_ready_pattern", "")
        if uboot_cmd:
            if not uboot_ready:
                uboot_ready = r"*=>"
            if not qemu_term.wait_pattern(uboot_ready, timeout=10.0):
                raise TerminalTimeoutError("timed out waiting for U-Boot prompt")
            qemu_term.send(uboot_cmd)
        if not qemu_term.wait_pattern(ZONE0_READY_PATTERN, timeout=180.0):
            raise TerminalTimeoutError("timed out waiting for zone0 shell prompt")
        return 0
    if cfg["mode"] == "board":
        log_path = logs_dir(cfg) / "zone0_console.log"
        log_path.write_text("", encoding="utf-8")

        board_term = build_terminal(cfg, log_path)
        board_term.open()
        cfg["_board_term"] = board_term
        board_power_cycle(cfg)
        time.sleep(3.0)
        board_wake_console(board_term)

        uboot_cmd = cfg.get("uboot_cmd", "")
        uboot_ready = cfg.get("uboot_ready_pattern", "")
        if uboot_cmd:
            if not uboot_ready:
                uboot_ready = r"=>"
            board_wait_uboot_prompt(board_term, uboot_ready, timeout=10.0)
            time.sleep(0.3)
            board_term.send(uboot_cmd)
        if not board_term.wait_pattern(ZONE0_READY_PATTERN, timeout=180.0):
            raise TerminalTimeoutError("timed out waiting for zone0 shell prompt")
        return 0
    return 0


def lspci_expected_cfg(cfg: dict[str, Any]) -> dict[str, Any]:
    raw = cfg.get("lspci") or {}
    if not isinstance(raw, dict):
        raw = {}
    expected_bdfs = [str(x) for x in raw.get("expected_bdfs") or []]
    min_count = int(raw.get("min_count", len(expected_bdfs) or 1))
    return {"expected_bdfs": expected_bdfs, "min_count": min_count}


def save_lspci_artifacts(cfg: dict[str, Any], name: str, content: str) -> None:
    path = logs_dir(cfg) / name
    path.write_text(content, encoding="utf-8")
    print(f"[lspci] saved {path}", flush=True)


def lspci(cfg: dict[str, Any], term: Terminal | None) -> int:
    print("————————————————\ncase: lspci\n————————————————\n", flush=True)
    if term is None:
        raise SystemExit("terminal backend is required (run zone0_start first)")

    expected = lspci_expected_cfg(cfg)

    _, output = term.run(
        "lspci",
        "timeout 20 lspci -D > /tmp/ci_lspci.log 2>&1; cat /tmp/ci_lspci.log",
        timeout=60.0,
    )
    save_lspci_artifacts(cfg, "lspci.log", f"=== lspci -D ===\n{output}\n")

    matched = [bdf for bdf in expected["expected_bdfs"] if bdf in output]
    missing = [bdf for bdf in expected["expected_bdfs"] if bdf not in output]
    line_count = len([line for line in output.splitlines() if line.strip()])

    print(f"[lspci] devices listed: {line_count}", flush=True)
    if expected["expected_bdfs"]:
        print(
            f"[lspci] matched expected BDFs ({len(matched)}/{len(expected['expected_bdfs'])}): {matched}",
            flush=True,
        )
        if missing:
            print(f"[lspci] missing expected BDFs: {missing}", flush=True)

    if expected["expected_bdfs"]:
        if len(matched) < expected["min_count"]:
            raise TerminalCommandError(
                "lspci found "
                f"{len(matched)}/{expected['min_count']} expected devices; "
                f"missing: {missing}"
            )
    elif line_count < expected["min_count"]:
        raise TerminalCommandError(
            f"lspci listed {line_count} devices, expected at least {expected['min_count']}"
        )

    print("lspci verification passed", flush=True)
    return 0


def ping_success(output: str) -> bool:
    if re.search(r"\b0% (?:packet )?loss\b", output):
        return True
    match = re.search(r"(\d+) packets? received", output)
    return bool(match and int(match.group(1)) > 0)


def network(cfg: dict[str, Any], term: Terminal | None) -> int:
    print("————————————————\ncase: network\n————————————————\n", flush=True)
    if cfg["mode"] != "board":
        print("[network] skipped (not board mode)", flush=True)
        return 0
    if term is None:
        raise SystemExit("terminal backend is required (run zone0_start first)")

    host_ip = cfg["network_host_ip"]
    ping_count = cfg["network_ping_count"]
    _, output = term.run(
        "network_ping",
        f"ping -c {ping_count} -W 5 {host_ip}",
        timeout=30.0,
    )
    save_lspci_artifacts(cfg, "network_ping.log", f"=== ping {host_ip} ===\n{output}\n")

    if not ping_success(output):
        raise TerminalCommandError(f"ping {host_ip} failed")

    print(f"[network] ping {host_ip} ok, staging files on host", flush=True)
    script = cfg["workspace"] / "jenkins" / "board_scp.sh"
    if not script.is_file():
        raise SystemExit(f"board scp script not found: {script}")

    env = os.environ.copy()
    env["ARCH"] = cfg["arch"]
    env["BOARD"] = cfg["board"]
    if cfg["kdir"]:
        env["KDIR"] = cfg["kdir"]
    env["WORKSPACE_ROOT"] = str(cfg["workspace"])
    env["HVISOR_TOOL_PATH"] = cfg["hvisor_tool_path"]
    env["STAGING_DIR"] = cfg["network_staging_dir"]
    if cfg.get("zone1_dtb"):
        zone1_dtb = Path(cfg["zone1_dtb"])
        if not zone1_dtb.is_absolute():
            zone1_dtb = cfg["workspace"] / zone1_dtb
        env["ZONE1_DTB"] = str(zone1_dtb.resolve())

    subprocess.run(["bash", str(script)], check=True, cwd=cfg["workspace"], env=env)

    # Let host staging finish and drain any buffered serial output before scp.
    time.sleep(3.0)

    host_user = cfg["network_host_user"]
    staging_dir = cfg["network_staging_dir"]
    term.run(
        "network_scp_env",
        f"export CI_H={host_ip} CI_U={host_user} CI_D={staging_dir}",
        timeout=30.0,
    )
    # Keep the scp line short: serial consoles truncate long commands.
    scp_cmd = "scp -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null $CI_U@$CI_H:$CI_D/* /root/"
    print(f"[network] pulling staged files from {host_user}@{host_ip}:{staging_dir}", flush=True)
    pull_rc, _ = term.run("network_scp_pull", scp_cmd, timeout=300.0)
    if pull_rc != 0:
        raise TerminalCommandError(f"board scp pull failed with rc={pull_rc}")

    term.run(
        "network_chmod",
        "chmod +x /root/boot_zone1.sh /root/check_serial.sh 2>/dev/null || true",
        timeout=15.0,
    )
    print("network test and file deploy passed", flush=True)
    return 0


def zone1_start(cfg: dict[str, Any], term: Terminal | None) -> int:
    print("————————————————\ncase: zone1_start\n————————————————\n", flush=True)
    if term is None:
        raise SystemExit("terminal backend is required")

    inner_log = "/tmp/zone1_inner.log"
    _, _ = term.run("zone1_cd", "cd /root", timeout=15.0)
    _, _ = term.run("zone1_ls", "ls", timeout=15.0)
    _, _ = term.run("zone1_cat_boot", "cat boot_zone1.sh", timeout=15.0)
    boot_rc, _ = term.run("zone1_boot", "./boot_zone1.sh", timeout=120.0)
    _, _ = term.run("zone1_list", "./hvisor zone list", timeout=15.0)

    max_pts = find_zone1_pts(term)

    check_rc, _ = term.run(
        "zone1_serial_check",
        f"./check_serial.sh /dev/pts/{max_pts} {inner_log} {int(ZONE1_INNER_PROMPT_TIMEOUT)}",
        timeout=ZONE1_INNER_PROMPT_TIMEOUT + 30.0,
    )
    # Fetch via tail to limit serial traffic; generous timeout for slow Jenkins consoles.
    _, inner_output = term.run(
        "zone1_inner_log",
        f"tail -c 131072 {inner_log}",
        timeout=ZONE1_INNER_LOG_FETCH_TIMEOUT,
    )
    save_inner_serial_log(cfg, inner_output)

    if boot_rc != 0:
        raise TerminalCommandError(f"command failed with rc={boot_rc}: ./boot_zone1.sh")
    if check_rc != 0:
        raise TerminalCommandError(
            f"command failed with rc={check_rc}: check_serial.sh (no console prompt)"
        )
    print("zone1_started successfully", flush=True)
    return 0


CASE_HANDLERS: dict[str, CaseFunc] = {
    "zone0_start": zone0_start,
    "network": network,
    "lspci": lspci,
    "zone1_start": zone1_start,
}


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Run BID test cases from jenkins/ci.yaml")
    parser.add_argument("--bid", required=True, help="BID key in jenkins/ci.yaml, e.g. aarch64/qemu-gicv3")
    return parser.parse_args()


def load_runtime_config(args: argparse.Namespace) -> dict[str, Any]:
    ci = load_ci()
    bid_entry = get_bid_entry(ci, args.bid)
    tests = bid_entry["tests"]

    try:
        arch, board = parse_bid(args.bid)
    except ValueError as exc:
        raise SystemExit(str(exc)) from exc
    mode = bid_entry.get("mode", "").strip()
    cases = bid_entry.get("cases", [])
    if not mode:
        raise SystemExit(f"incomplete config for bid '{args.bid}': tests.mode is required")
    if not cases:
        raise SystemExit(f"no test cases configured for bid '{args.bid}'")

    cell_root = Path.cwd()
    build_args = bid_entry.get("build_args") or {}
    network_cfg = tests.get("network") or {}
    if not isinstance(network_cfg, dict):
        network_cfg = {}
    hvisor_tool_path = os.environ.get("HVISOR_TOOL_PATH", "").strip()
    if not hvisor_tool_path:
        hvisor_tool_path = str((cell_root / "hvisor-tool").resolve())
    elif not Path(hvisor_tool_path).is_absolute():
        hvisor_tool_path = str((cell_root / hvisor_tool_path).resolve())
    return {
        "bid": args.bid,
        "arch": arch,
        "board": board,
        "mode": mode,
        "cases": cases,
        "workspace": cell_root,
        "kdir": str(build_args.get("KDIR", "")).strip(),
        "hvisor_tool_path": hvisor_tool_path,
        "socket_path": str((cell_root / ".qemu" / "qemu.sock").resolve()),
        "serial_port": str(tests.get("serial", "/dev/null")),
        "power_serial": str(tests.get("power_serial", "")).strip(),
        "baudrate": int(tests.get("baudrate", 1500000)),
        "uboot_cmd": str(tests.get("uboot_cmd", "")).strip(),
        "uboot_ready_pattern": str(tests.get("uboot_ready_pattern", "")).strip(),
        "tftp_dir": str(tests.get("tftp_dir", "/home/light/tftp")).strip(),
        "lspci": tests.get("lspci") or {},
        "network_host_ip": str(network_cfg.get("host_ip", "192.168.1.181")).strip(),
        "network_host_user": str(network_cfg.get("host_user", "light")).strip(),
        "network_staging_dir": str(
            network_cfg.get("staging_dir", "/home/light/tftp/ci_deploy")
        ).strip(),
        "network_ping_count": int(network_cfg.get("ping_count", 3)),
        "zone1_dtb": str(network_cfg.get("zone1_dtb", "")).strip(),
    }


def build_terminal(cfg: dict[str, Any], log_path: Path) -> Terminal:
    if cfg["mode"] == "qemu":
        return Terminal.from_qemu_socket(path=cfg["socket_path"], log_path=log_path)
    return Terminal.from_serial(port=cfg["serial_port"], baudrate=cfg["baudrate"], log_path=log_path)


def main() -> int:
    args = parse_args()
    cfg = load_runtime_config(args)
    try:
        for case_name in cfg["cases"]:
            case_fn = CASE_HANDLERS.get(case_name)
            if case_fn is None:
                available = ", ".join(sorted(CASE_HANDLERS.keys()))
                raise SystemExit(f"unknown case '{case_name}', available: {available}")

            if case_name == "zone0_start":
                rc = case_fn(cfg, None)
                if rc != 0:
                    return rc
                time.sleep(5.0)
                continue

            term = get_active_terminal(cfg)
            if term is None:
                log_path = logs_dir(cfg) / "zone1_console.log"
                log_path.write_text("", encoding="utf-8")
                with build_terminal(cfg, log_path) as term:
                    try:
                        rc = case_fn(cfg, term)
                    except (TerminalTimeoutError, TerminalCommandError) as exc:
                        print(f"[ci_runner] terminal command failed in case '{case_name}': {exc}", flush=True)
                        return 1
                    if rc != 0:
                        return rc
            else:
                try:
                    rc = case_fn(cfg, term)
                except (TerminalTimeoutError, TerminalCommandError) as exc:
                    print(f"[ci_runner] terminal command failed in case '{case_name}': {exc}", flush=True)
                    return 1
                if rc != 0:
                    return rc
            time.sleep(5.0)
        return 0
    finally:
        close_active_terminal(cfg)
        terminate_managed_process(cfg)


if __name__ == "__main__":
    raise SystemExit(main())
