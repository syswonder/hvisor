#!/usr/bin/env python3
from __future__ import annotations

import argparse
import os
import re
import socket
import signal
import shutil
import subprocess
import time
from pathlib import Path
from typing import Any, Callable

from ci_config import get_bid_entry, load_ci, parse_bid
from terminal import Terminal, TerminalCommandError, TerminalTimeoutError


CaseFunc = Callable[[dict[str, Any], Terminal | None], int]
ZONE_ID_PATTERN = re.compile(r"(?m)^\s*1\s+")


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


def run_and_print(term: Terminal, command: str) -> str:
    output = term.send_until_quiet(command, quiet_seconds=1.0, max_duration=40.0)
    if output:
        print(output, end="", flush=True)
    return output


def run_and_print_quiet(
    term: Terminal,
    command: str,
    quiet_seconds: float = 1.0,
    max_duration: float = 30.0,
    check_exit: bool = True,
) -> tuple[str, int]:
    # Send a leading Enter to synchronize shell prompt state.
    # term.send("\n")
    # _ = term.read_for(duration=0.2)

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
    # For non-shell environments (e.g. U-Boot), do not append shell-style
    # status markers; just send and wait for output to go quiet.
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
    # Read side is decoupled from send side for interactive boot flows.
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
    # For commands that switch interactive context (e.g. screen attach),
    # only send and collect a short best-effort echo.
    output = term.send_and_drain(command, read_duration=read_duration)
    if output:
        print(output, end="", flush=True)
    return output


def wait_for_pattern(
    term: Terminal,
    pattern: str,
    timeout: float,
    *,
    flags: int = 0,
    echo: bool = True,
) -> str:
    regex = re.compile(pattern, flags)
    deadline = time.monotonic() + timeout
    buf = ""
    while time.monotonic() < deadline:
        chunk = term.backend.read()
        if chunk:
            text = chunk.decode(term.encoding, errors="replace")
            buf += text
            if echo:
                print(text, end="", flush=True)
            if regex.search(buf):
                return buf
            continue
        time.sleep(0.05)
    raise TerminalTimeoutError(f"timed out waiting for pattern: {pattern}")


def send_and_wait(
    term: Terminal,
    command: str,
    pattern: str,
    timeout: float,
    *,
    flags: int = 0,
) -> str:
    term.send(command)
    return wait_for_pattern(term, pattern, timeout, flags=flags)


def ensure_board_login(cfg: dict[str, Any], term: Terminal, timeout: float = 90.0) -> None:
    term.send("")
    output = wait_for_pattern(
        term,
        r"(Phytium-Pi login:|[$#]\s*$)",
        timeout,
        flags=re.MULTILINE,
    )
    if re.search(r"[$#]\s*$", output, re.MULTILINE):
        return
    send_and_wait(term, cfg["board_user"], r"Password:", 15.0)
    send_and_wait(term, cfg["board_pass"], r"[$#]\s*$", 30.0, flags=re.MULTILINE)


def remote_command_prefix(cfg: dict[str, Any]) -> list[str]:
    target = f"{cfg['board_user']}@{cfg['board_ip']}"
    base = [
        "ssh",
        "-o",
        "StrictHostKeyChecking=no",
        "-o",
        "UserKnownHostsFile=/dev/null",
    ]
    if cfg.get("board_pass") and shutil.which("sshpass"):
        return ["sshpass", "-p", cfg["board_pass"], *base, target]
    return [*base, target]


def scp_command_prefix(cfg: dict[str, Any]) -> list[str]:
    base = [
        "scp",
        "-o",
        "StrictHostKeyChecking=no",
        "-o",
        "UserKnownHostsFile=/dev/null",
    ]
    if cfg.get("board_pass") and shutil.which("sshpass"):
        return ["sshpass", "-p", cfg["board_pass"], *base]
    return base


def run_host_command(cmd: list[str], timeout: float = 60.0) -> None:
    print("+ " + " ".join(cmd), flush=True)
    subprocess.run(cmd, check=True, timeout=timeout)


def set_board_ip(cfg: dict[str, Any], term: Terminal) -> None:
    cmd = (
        f"echo {cfg['board_pass']} | sudo -S "
        f"ifconfig {cfg['board_iface']} {cfg['board_ip']}"
    )
    _, _ = run_and_print_quiet(term, cmd, quiet_seconds=1.0, max_duration=20.0)


def clear_board_zone1_dir(cfg: dict[str, Any]) -> None:
    remote_cmd = f"mkdir -p '{cfg['scp_dst']}' && find '{cfg['scp_dst']}' -mindepth 1 -maxdepth 1 -exec rm -rf {{}} +"
    run_host_command([*remote_command_prefix(cfg), remote_cmd], timeout=60.0)


def copy_zone1_files(cfg: dict[str, Any]) -> None:
    src = Path(cfg["scp_src"])
    required = [
        "hvisor",
        "hvisor.ko",
        "linux2.dtb",
        "Image",
        "rootfs2.ext4",
        "start.sh",
        "zone1-linux-virtio.json",
        "zone1-linux.json",
    ]
    missing = [name for name in required if not (src / name).is_file()]
    if missing:
        raise SystemExit(f"missing zone1 staging files in {src}: {', '.join(missing)}")
    target = f"{cfg['board_user']}@{cfg['board_ip']}:{cfg['scp_dst']}/"
    run_host_command([*scp_command_prefix(cfg), *[str(src / name) for name in required], target], timeout=180.0)


def phytium_deploy_zone1(cfg: dict[str, Any], term: Terminal | None) -> int:
    print("————————————————\ncase: phytium_deploy_zone1\n————————————————\n", flush=True)
    if term is None:
        raise SystemExit("terminal backend is required")
    ensure_board_login(cfg, term)
    set_board_ip(cfg, term)
    clear_board_zone1_dir(cfg)
    copy_zone1_files(cfg)
    print("zone1 files deployed successfully", flush=True)
    return 0


def phytium_boot_zone0(cfg: dict[str, Any], term: Terminal | None) -> int:
    print("————————————————\ncase: phytium_boot_zone0\n————————————————\n", flush=True)
    if term is None:
        raise SystemExit("terminal backend is required")
    print("Reset or power-cycle the Phytium-Pi board now.", flush=True)
    wait_for_pattern(
        term,
        r"(Hit any key to stop autoboot|Autoboot|Phytium-Pi#)",
        timeout=float(cfg["uboot_wait_timeout"]),
    )
    term.send("")
    wait_for_pattern(term, r"Phytium-Pi#", timeout=20.0)
    run_and_print_quiet_raw(
        term,
        cfg["uboot_cmd"],
        quiet_seconds=3.0,
        max_duration=float(cfg["zone0_boot_timeout"]),
    )
    return 0


def phytium_start_zone1(cfg: dict[str, Any], term: Terminal | None) -> int:
    print("————————————————\ncase: phytium_start_zone1\n————————————————\n", flush=True)
    if term is None:
        raise SystemExit("terminal backend is required")
    ensure_board_login(cfg, term, timeout=float(cfg["zone0_login_timeout"]))
    _, _ = run_and_print_quiet(term, f"cd {cfg['scp_dst']}", quiet_seconds=1.0, max_duration=15.0)
    _, _ = run_and_print_quiet(term, "chmod +x start.sh", quiet_seconds=1.0, max_duration=15.0)
    _, _ = run_and_print_quiet(term, "./start.sh", quiet_seconds=5.0, max_duration=60.0)
    time.sleep(float(cfg["zone1_start_wait"]))
    return 0


def phytium_check_zone(cfg: dict[str, Any], term: Terminal | None) -> int:
    print("————————————————\ncase: phytium_check_zone\n————————————————\n", flush=True)
    if term is None:
        raise SystemExit("terminal backend is required")
    _, _ = run_and_print_quiet(term, f"cd {cfg['scp_dst']}", quiet_seconds=1.0, max_duration=15.0)
    output, _ = run_and_print_quiet(term, "./hvisor zone list", quiet_seconds=1.0, max_duration=20.0)
    if not ZONE_ID_PATTERN.search(output):
        raise TerminalCommandError("zone id 1 was not found in './hvisor zone list' output")
    print("phytium-pi zone1 check passed", flush=True)
    return 0


def zone0_start(cfg: dict[str, Any], term: Terminal | None) -> int:
    print("————————————————\ncase: zone0_start\n————————————————\n", flush=True)
    if cfg["mode"] == "qemu":
        cmd = ["make", f"ARCH={cfg['arch']}", f"BOARD={cfg['board']}", "MODE=release", "ci-run"]
        proc = subprocess.Popen(cmd, cwd=cfg["workspace"], start_new_session=True)
        cfg["_managed_proc"] = proc
        cfg["_managed_proc_name"] = "qemu ci-run"
        wait_qemu_socket(cfg["socket_path"], timeout=30.0)
        with build_terminal(cfg) as qemu_term:
            bid = cfg["bid"]
            if bid == "aarch64/qemu-gicv3":
                _ = read_and_print_until_quiet(
                    qemu_term,
                    quiet_seconds=3.0,
                    max_duration=10.0,
                )
                qemu_term.send("bootm 0x40400000 - 0x40000000")
            if bid == "x86_64/qemu":
                time.sleep(10.0)
            _ = read_and_print_until_quiet(
                qemu_term,
                quiet_seconds=5,
                max_duration=180.0,
            )
        return 0
    if cfg["mode"] == "board":
        # TODO: reboot board
        return 0
    return 0


def zone1_start(cfg: dict[str, Any], term: Terminal | None) -> int:
    print("————————————————\ncase: zone1_start\n————————————————\n", flush=True)
    if term is None:
        raise SystemExit("terminal backend is required")
    # _ = run_and_print_quiet_raw(term, "bash", quiet_seconds=1.0, max_duration=15.0)
    _, _ = run_and_print_quiet(term, "cd /root", quiet_seconds=1.0, max_duration=15.0)
    _, _ = run_and_print_quiet(term, "ls", quiet_seconds=1.0, max_duration=15.0)
    _, _ = run_and_print_quiet(term, "cat boot_zone1.sh", quiet_seconds=1.0, max_duration=15.0)
    _, boot_rc = run_and_print_quiet(
        term,
        "./boot_zone1.sh",
        quiet_seconds=15,
        max_duration=30.0,
    )
    _, _ = run_and_print_quiet(term, "./hvisor zone list", quiet_seconds=1.0, max_duration=15.0)
    if cfg["arch"] != "x86_64":
        _ = run_and_print_quiet_raw(term, "script /dev/null", quiet_seconds=1.0, max_duration=15.0)
    pts_output, _ = run_and_print_quiet(term, "ls -1 /dev/pts/[0-9]*", quiet_seconds=1.0, max_duration=15.0)
    pts_numbers = sorted(int(match) for match in re.findall(r"/dev/pts/(\d+)", pts_output))
    if not pts_numbers:
        raise TerminalCommandError("failed to find numeric pts from 'ls -1 /dev/pts/[0-9]*'")
    max_pts = pts_numbers[-1]
    _ = run_and_print_send_only(term, f"screen /dev/pts/{max_pts}", read_duration=20.0)
    _ = run_and_print_send_only(term, "\n", read_duration=2.0)
    _, _ = run_and_print_quiet(term, "ls", quiet_seconds=1.0, max_duration=15.0)
    if boot_rc != 0:
        raise TerminalCommandError(f"command failed with rc={boot_rc}: sh ./boot_zone1.sh")
    else:
        print("zone1_started successfully", flush=True)
    return 0


CASE_HANDLERS: dict[str, CaseFunc] = {
    "phytium_boot_zone0": phytium_boot_zone0,
    "phytium_check_zone": phytium_check_zone,
    "phytium_deploy_zone1": phytium_deploy_zone1,
    "phytium_start_zone1": phytium_start_zone1,
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

    cfg = {
        "bid": args.bid,
        "arch": arch,
        "board": board,
        "mode": mode,
        "cases": cases,
        "workspace": Path(__file__).resolve().parent.parent,
        "socket_path": str((Path(__file__).resolve().parent.parent / ".qemu" / "qemu.sock").resolve()),
        "serial_port": str(tests.get("serial", "/dev/null")),
        "baudrate": int(tests.get("baudrate", 1500000)),
        "board_ip": str(tests.get("board_ip", "")),
        "server_ip": str(tests.get("server_ip", "")),
        "board_user": str(tests.get("board_user", "user")),
        "board_pass": str(tests.get("board_pass", "")),
        "board_iface": str(tests.get("board_iface", "eth0")),
        "scp_src": str(tests.get("scp_src", "")),
        "scp_dst": str(tests.get("scp_dst", "/home/user/zone1")),
        "uboot_cmd": str(tests.get("uboot_cmd", "run boot_root_linux")),
        "uboot_wait_timeout": float(tests.get("uboot_wait_timeout", 300.0)),
        "zone0_boot_timeout": float(tests.get("zone0_boot_timeout", 180.0)),
        "zone0_login_timeout": float(tests.get("zone0_login_timeout", 180.0)),
        "zone1_start_wait": float(tests.get("zone1_start_wait", 30.0)),
    }
    if mode == "board":
        required = ["serial_port", "board_ip", "board_user", "scp_src", "scp_dst", "uboot_cmd"]
        missing = [key for key in required if not str(cfg.get(key, "")).strip()]
        if missing:
            raise SystemExit(f"incomplete board config for bid '{args.bid}': missing {', '.join(missing)}")
    return cfg


def build_terminal(cfg: dict[str, Any]) -> Terminal:
    if cfg["mode"] == "qemu":
        return Terminal.from_qemu_socket(path=cfg["socket_path"])
    return Terminal.from_serial(port=cfg["serial_port"], baudrate=cfg["baudrate"])


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
            
            with build_terminal(cfg) as term:
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
        terminate_managed_process(cfg)


if __name__ == "__main__":
    raise SystemExit(main())
