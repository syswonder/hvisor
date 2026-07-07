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

# Wait for an interactive shell prompt, not login: (getty shows that before MOTD/shell).
ZONE0_READY_PATTERN = r"root@[^\r\n]*[#$]\s|(?:\r?\n)#\s"
ZONE1_INNER_PROMPT_TIMEOUT = 180.0


def bid_log_key(bid: str) -> str:
    return bid.replace("/", "__")


def jenkins_workspace_root(fallback: Path) -> Path:
    workspace = os.environ.get("WORKSPACE", "").strip()
    if workspace:
        return Path(workspace)
    return fallback


def logs_dir(cfg: dict[str, Any]) -> Path:
    root = jenkins_workspace_root(cfg["workspace"])
    path = root / "logs" / bid_log_key(cfg["bid"])
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


def save_inner_serial_log(cfg: dict[str, Any], content: str) -> None:
    if not content:
        return
    path = logs_dir(cfg) / "zone1_inner_serial.log"
    path.write_text(content, encoding="utf-8")


def zone0_start(cfg: dict[str, Any], term: Terminal | None) -> int:
    print("————————————————\ncase: zone0_start\n————————————————\n", flush=True)
    if cfg["mode"] == "qemu":
        cmd = ["make", f"ARCH={cfg['arch']}", f"BOARD={cfg['board']}", "MODE=release", "ci-run"]
        proc = subprocess.Popen(cmd, cwd=cfg["workspace"], start_new_session=True)
        cfg["_managed_proc"] = proc
        cfg["_managed_proc_name"] = "qemu ci-run"
        wait_qemu_socket(cfg["socket_path"], timeout=30.0)

        log_path = logs_dir(cfg) / "zone0_console.log"
        log_path.write_text("", encoding="utf-8")

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

        uboot_cmd = cfg.get("uboot_cmd", "")
        uboot_ready = cfg.get("uboot_ready_pattern", "")
        if uboot_cmd:
            if not uboot_ready:
                uboot_ready = r"Net:.*\n=> "
            if not board_term.wait_pattern(uboot_ready, timeout=120.0):
                raise TerminalTimeoutError("timed out waiting for U-Boot prompt")
            time.sleep(0.3)
            board_term.flush_input()
            board_term.send(uboot_cmd)
        if not board_term.wait_pattern(ZONE0_READY_PATTERN, timeout=180.0):
            raise TerminalTimeoutError("timed out waiting for zone0 shell prompt")
        return 0
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
    _, inner_output = term.run("zone1_inner_log", f"cat {inner_log}", timeout=15.0)
    save_inner_serial_log(cfg, inner_output)

    if boot_rc != 0:
        raise TerminalCommandError(f"command failed with rc={boot_rc}: ./boot_zone1.sh")
    if check_rc != 0:
        raise TerminalCommandError(
            f"command failed with rc={check_rc}: check_serial.sh (no shell prompt)"
        )
    print("zone1_started successfully", flush=True)
    return 0


CASE_HANDLERS: dict[str, CaseFunc] = {
    "zone0_start": zone0_start,
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
    return {
        "bid": args.bid,
        "arch": arch,
        "board": board,
        "mode": mode,
        "cases": cases,
        "workspace": cell_root,
        "socket_path": str((cell_root / ".qemu" / "qemu.sock").resolve()),
        "serial_port": str(tests.get("serial", "/dev/null")),
        "power_serial": str(tests.get("power_serial", "")).strip(),
        "baudrate": int(tests.get("baudrate", 1500000)),
        "uboot_cmd": str(tests.get("uboot_cmd", "")).strip(),
        "uboot_ready_pattern": str(tests.get("uboot_ready_pattern", "")).strip(),
        "tftp_dir": str(tests.get("tftp_dir", "/home/light/tftp")).strip(),
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
