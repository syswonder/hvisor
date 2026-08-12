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

from board_flow import (
    board_login,
    board_network_and_trans,
    board_power_off,
    board_zone1_start,
    boot_board_zone0_with_retry,
    close_board_terminal,
    get_board_terminal,
    logs_dir,
    release_logs_ownership,
)
from ci_config import get_bid_entry, load_ci, parse_bid
from terminal import Terminal, TerminalCommandError, TerminalTimeoutError


CaseFunc = Callable[[dict[str, Any], Terminal | None], int]


def platform_board(board: str, configured: str = "") -> str:
    return configured.strip() or board


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


def zone_list_shows_running(output: str, zone_name: str = "linux2") -> None:
    """hvisor-tool zone list returns zone count as exit code; validate output instead."""
    if zone_name not in output or "running" not in output:
        raise TerminalCommandError(
            f"zone list missing running {zone_name!r}:\n{output.strip()}"
        )


def run_and_print_quiet(
    term: Terminal,
    command: str,
    quiet_seconds: float = 1.0,
    max_duration: float = 30.0,
    check_exit: bool = True,
) -> tuple[str, int]:
    output, rc = term.run_until_quiet_with_status(
        command,
        quiet_seconds=quiet_seconds,
        max_duration=max_duration,
    )
    if output:
        print(output, end="", flush=True)
    if check_exit and rc != 0:
        raise TerminalCommandError(f"command failed with rc={rc}: {command}")
    return output, rc


def run_and_print_quiet_raw(
    term: Terminal,
    command: str,
    quiet_seconds: float = 1.0,
    max_duration: float = 30.0,
) -> str:
    output = term.send_until_quiet(
        command,
        quiet_seconds=quiet_seconds,
        max_duration=max_duration,
    )
    if output:
        print(output, end="", flush=True)
    return output


def read_and_print_until_quiet(
    term: Terminal,
    quiet_seconds: float = 3.0,
    max_duration: float = 120.0,
) -> str:
    output = term.read_until_quiet(
        quiet_seconds=quiet_seconds,
        max_duration=max_duration,
    )
    if output:
        print(output, end="", flush=True)
    return output


def run_and_print_send_only(
    term: Terminal,
    command: str,
    read_duration: float = 0.5,
) -> str:
    output = term.send_and_drain(command, read_duration=read_duration)
    if output:
        print(output, end="", flush=True)
    return output


def resolve_staging_dir(raw: str, arch: str, board: str) -> str:
    base = (raw or "/home/light/ci_deploy").rstrip("/")
    suffix = f"{arch}__{board.replace('/', '__')}"
    if base.endswith(suffix):
        return base
    return f"{base}/{suffix}"


def zone0_start(cfg: dict[str, Any], term: Terminal | None) -> int:
    print("————————————————\ncase: zone0_start\n————————————————\n", flush=True)
    if cfg["mode"] == "qemu":
        cmd = ["make", f"ARCH={cfg['arch']}", f"BOARD={cfg['board']}", "MODE=release", "ci-run"]
        env = os.environ.copy()
        env["BID"] = ""
        proc = subprocess.Popen(cmd, cwd=cfg["workspace"], start_new_session=True, env=env)
        cfg["_managed_proc"] = proc
        wait_qemu_socket(cfg["socket_path"], timeout=30.0)
        with build_terminal(cfg) as qemu_term:
            bid = cfg["bid"]
            if bid == "aarch64/qemu-gicv3":
                _ = read_and_print_until_quiet(
                    qemu_term,
                    quiet_seconds=3.0,
                    max_duration=10.0,
                )
                uboot = str(cfg.get("uboot_cmd", "")).strip()
                if uboot:
                    qemu_term.send(uboot)
                else:
                    qemu_term.send("bootm 0x40400000 - 0x40000000")
            if bid in ("x86_64/qemu", "x86_64/qemu_asterinas"):
                time.sleep(10.0)
            _ = read_and_print_until_quiet(
                qemu_term,
                quiet_seconds=5,
                max_duration=180.0,
            )
        return 0
    if cfg["mode"] == "board":
        log_path = logs_dir(cfg) / "zone0_console.log"
        log_path.write_text("", encoding="utf-8")
        board_term = build_terminal(cfg, log_path)
        board_term.open()
        cfg["_board_term"] = board_term
        boot_board_zone0_with_retry(cfg, board_term)
        return 0
    return 0


def login(cfg: dict[str, Any], term: Terminal | None) -> int:
    print("————————————————\ncase: login\n————————————————\n", flush=True)
    if cfg["mode"] != "board":
        print("[login] skipped (not board mode)", flush=True)
        return 0
    if term is None:
        raise SystemExit("terminal backend is required")
    return board_login(cfg, term)


def network_and_trans(cfg: dict[str, Any], term: Terminal | None) -> int:
    print("————————————————\ncase: network_and_trans\n————————————————\n", flush=True)
    if cfg["mode"] != "board":
        print("[network_and_trans] skipped (not board mode)", flush=True)
        return 0
    if term is None:
        raise SystemExit("terminal backend is required")
    return board_network_and_trans(cfg, term)


def zone1_start(cfg: dict[str, Any], term: Terminal | None) -> int:
    print("————————————————\ncase: zone1_start\n————————————————\n", flush=True)
    if cfg["mode"] == "board":
        if term is None:
            raise SystemExit("terminal backend is required")
        return board_zone1_start(cfg, term)
    if term is None:
        raise SystemExit("terminal backend is required")
    _, _ = run_and_print_quiet(term, "cd /root", quiet_seconds=1.0, max_duration=15.0)
    _, _ = run_and_print_quiet(
        term,
        "./boot_zone1.sh",
        quiet_seconds=15,
        max_duration=120.0,
        check_exit=False,
    )
    zone_list_out, _ = run_and_print_quiet(
        term,
        "./hvisor zone list",
        quiet_seconds=1.0,
        max_duration=15.0,
        check_exit=False,
    )
    zone_list_shows_running(zone_list_out, str(cfg.get("zone1_name", "linux2")))
    if cfg["arch"] != "x86_64":
        _ = run_and_print_quiet_raw(term, "script /dev/null", quiet_seconds=1.0, max_duration=15.0)
    pts_output, _ = run_and_print_quiet(term, "ls -1 /dev/pts/[0-9]*", quiet_seconds=1.0, max_duration=15.0)
    pts_numbers = sorted(int(match) for match in re.findall(r"/dev/pts/(\d+)", pts_output))
    if not pts_numbers:
        raise TerminalCommandError("failed to find numeric pts from 'ls -1 /dev/pts/[0-9]*'")
    max_pts = pts_numbers[-1]
    _ = run_and_print_send_only(term, f"screen /dev/pts/{max_pts}", read_duration=20.0)
    _ = run_and_print_send_only(term, "\n", read_duration=2.0)
    print("zone1_started successfully", flush=True)
    return 0


def asterinas_zone1_regression(cfg: dict[str, Any], term: Terminal | None) -> int:
    print("————————————————\ncase: asterinas_zone1_regression\n————————————————\n", flush=True)
    if term is None:
        raise SystemExit("terminal backend is required")

    _, _ = run_and_print_quiet(term, "cd /root", quiet_seconds=1.0, max_duration=15.0)
    _, _ = run_and_print_quiet(
        term,
        "cat boot_zone1_asterinas.sh",
        quiet_seconds=1.0,
        max_duration=15.0,
    )
    _, _ = run_and_print_quiet(
        term,
        "bash boot_zone1_asterinas.sh",
        quiet_seconds=15,
        max_duration=45.0,
    )
    zone_list_out, _ = run_and_print_quiet(
        term,
        "./hvisor zone list",
        quiet_seconds=1.0,
        max_duration=15.0,
        check_exit=False,
    )
    zone_list_shows_running(zone_list_out, "asterinas")

    pts_output, _ = run_and_print_quiet(
        term,
        "ls -1 /dev/pts/[0-9]*",
        quiet_seconds=1.0,
        max_duration=15.0,
    )
    pts_numbers = sorted(int(match) for match in re.findall(r"/dev/pts/(\d+)", pts_output))
    if not pts_numbers:
        raise TerminalCommandError("failed to find numeric pts from 'ls -1 /dev/pts/[0-9]*'")
    max_pts = pts_numbers[-1]

    _ = run_and_print_send_only(term, f"screen /dev/pts/{max_pts}", read_duration=20.0)
    _ = read_and_print_until_quiet(term, quiet_seconds=3.0, max_duration=30.0)

    regression_marker = "__HV_REGRESSION_RC_"
    term.send_one_by_one(f"/test/run_regression_test.sh; echo {regression_marker}$?")
    output = ""
    deadline = time.monotonic() + 900.0
    while time.monotonic() < deadline:
        chunk = term.read_for(duration=2.0)
        if not chunk:
            continue
        output += chunk
        print(chunk, end="", flush=True)

        rc_match = re.search(re.escape(regression_marker) + r"(\d+)", output)
        if rc_match is not None and rc_match.group(1) != "0":
            raise TerminalCommandError(
                f"Asterinas regression exited with rc={rc_match.group(1)}"
            )
        if rc_match is not None and "All regression tests passed" in output:
            break
    if "All regression tests passed" not in output:
        raise TerminalCommandError("Asterinas regression completion marker not found")

    # Ctrl-A d detaches from GNU screen; Ctrl-A Ctrl-A d is not a detach.
    term.backend.write(b"\x01d")
    time.sleep(1.0)
    term.backend.write(b"\r")
    detach_output = read_and_print_until_quiet(term, quiet_seconds=2.0, max_duration=10.0)
    if "root@zone0" not in detach_output:
        raise TerminalCommandError("failed to return to zone0 after screen detach")
    print("asterinas_zone1_regression_passed", flush=True)
    return 0


CASE_HANDLERS: dict[str, CaseFunc] = {
    "zone0_start": zone0_start,
    "login": login,
    "network_and_trans": network_and_trans,
    "zone1_start": zone1_start,
    "asterinas_zone1_regression": asterinas_zone1_regression,
}


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Run BID test cases from jenkins/ci.yaml")
    parser.add_argument("--bid", required=True, help="BID key in jenkins/ci.yaml")
    return parser.parse_args()


def load_runtime_config(args: argparse.Namespace) -> dict[str, Any]:
    ci = load_ci()
    bid_entry = get_bid_entry(ci, args.bid)
    tests = bid_entry["tests"]
    deploy = tests.get("deploy") or {}
    if not isinstance(deploy, dict):
        deploy = {}

    try:
        arch, bid_board = parse_bid(args.bid)
    except ValueError as exc:
        raise SystemExit(str(exc)) from exc
    board = platform_board(bid_board, bid_entry.get("platform_board", ""))
    mode = bid_entry.get("mode", "").strip()
    cases = bid_entry.get("cases", [])
    if not mode:
        raise SystemExit(f"incomplete config for bid '{args.bid}': tests.mode is required")
    if not cases:
        raise SystemExit(f"no test cases configured for bid '{args.bid}'")

    cell_root = Path.cwd()
    hvisor_tool_path = os.environ.get("HVISOR_TOOL_PATH", "").strip()
    if not hvisor_tool_path:
        hvisor_tool_path = str((cell_root / "hvisor-tool").resolve())
    elif not Path(hvisor_tool_path).is_absolute():
        hvisor_tool_path = str((cell_root / hvisor_tool_path).resolve())

    scp_tmp_dir = str(deploy.get("scp_tmp_dir", "/tmp/ci_pull")).strip() or "/tmp/ci_pull"
    uboot_step = float(tests.get("uboot_step_timeout", 0.0))
    uboot_prompt = float(tests.get("uboot_prompt_timeout", 60.0))
    if uboot_step <= 0:
        uboot_step = max(uboot_prompt, 60.0)

    return {
        "bid": args.bid,
        "arch": arch,
        "board": board,
        "mode": mode,
        "cases": cases,
        "workspace": cell_root,
        "hvisor_tool_path": hvisor_tool_path,
        "socket_path": str((cell_root / ".qemu" / "qemu.sock").resolve()),
        "serial_port": str(tests.get("serial", "/dev/null")),
        "power_serial": str(tests.get("power_serial", "")).strip(),
        "power_channel": int(tests.get("power_channel", 4)),
        "baudrate": int(tests.get("baudrate", 1500000)),
        "uboot_cmd": str(tests.get("uboot_cmd", "")).strip(),
        "uboot_cmds": tests.get("uboot_cmds") or [],
        "uboot_ready_pattern": str(tests.get("uboot_ready_pattern", "")).strip(),
        "uboot_prompt_timeout": uboot_prompt,
        "uboot_step_timeout": uboot_step,
        "uboot_autoboot_window": float(tests.get("uboot_autoboot_window", 0.0)),
        "zone0_ready_pattern": str(tests.get("zone0_ready_pattern", "")).strip(),
        "zone0_shell_timeout": float(tests.get("zone0_shell_timeout", 180.0)),
        "board_ip": str(deploy.get("board_ip", "")).strip(),
        "board_iface": str(deploy.get("board_iface", "eth0")).strip(),
        "board_netmask": str(deploy.get("board_netmask", "255.255.255.0")).strip(),
        "host_ip": str(deploy.get("host_ip", "")).strip(),
        "host_user": str(deploy.get("host_user", "light")).strip(),
        "link_wait": float(deploy.get("link_wait", 0.0)),
        "ping_retries": int(deploy.get("ping_retries", 1)),
        "ping_count": int(deploy.get("ping_count", 3)),
        "ssh_connect_timeout": int(deploy.get("ssh_connect_timeout", 60)),
        "pull_timeout": float(deploy.get("pull_timeout", 600.0)),
        "staging_dir": resolve_staging_dir(
            str(deploy.get("staging_dir", "/home/light/ci_deploy")),
            arch,
            board,
        ),
        "zone1_work_dir": str(deploy.get("zone1_work_dir", "/root")).strip() or "/root",
        "zone1_dtb": str(deploy.get("zone1_dtb", "")).strip(),
        "scp_tmp_dir": scp_tmp_dir,
        "scp_tmp_file": f"{scp_tmp_dir}/f",
        "retry_zone0": 2,
        "retry_network": 2,
        "retry_zone1": 1,
    }


def build_terminal(cfg: dict[str, Any], log_path: Path | None = None) -> Terminal:
    if cfg["mode"] == "qemu":
        return Terminal.from_qemu_socket(path=cfg["socket_path"], log_path=log_path)
    return Terminal.from_serial(
        port=cfg["serial_port"],
        baudrate=cfg["baudrate"],
        log_path=log_path,
    )


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
                try:
                    rc = case_fn(cfg, None)
                except (TerminalTimeoutError, TerminalCommandError) as exc:
                    print(f"[ci_runner] failed in case '{case_name}': {exc}", flush=True)
                    return 1
                if rc != 0:
                    return rc
                time.sleep(5.0)
                continue

            term = get_board_terminal(cfg) if cfg["mode"] == "board" else None
            if term is None and cfg["mode"] == "board":
                log_path = logs_dir(cfg) / "board_console.log"
                log_path.write_text("", encoding="utf-8")
                term = build_terminal(cfg, log_path)
                term.open()
                cfg["_board_term"] = term

            if cfg["mode"] == "qemu":
                with build_terminal(cfg) as qemu_term:
                    try:
                        rc = case_fn(cfg, qemu_term)
                    except (TerminalTimeoutError, TerminalCommandError) as exc:
                        print(f"[ci_runner] failed in case '{case_name}': {exc}", flush=True)
                        return 1
                    if rc != 0:
                        return rc
            else:
                assert term is not None
                try:
                    rc = case_fn(cfg, term)
                except (TerminalTimeoutError, TerminalCommandError) as exc:
                    print(f"[ci_runner] failed in case '{case_name}': {exc}", flush=True)
                    return 1
                if rc != 0:
                    return rc
            time.sleep(5.0)
        return 0
    finally:
        release_logs_ownership(cfg)
        close_board_terminal(cfg)
        terminate_managed_process(cfg)
        if cfg.get("mode") == "board":
            board_power_off(cfg)


if __name__ == "__main__":
    raise SystemExit(main())
