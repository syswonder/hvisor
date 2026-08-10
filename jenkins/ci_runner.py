#!/usr/bin/env python3
from __future__ import annotations

import argparse
import base64
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


def board_power(cfg: dict[str, Any], action: str) -> None:
    power_port = str(cfg.get("power_serial", "")).strip()
    if not power_port:
        print(f"[power] skip {action}: power_serial is empty", flush=True)
        return
    script = board_power_script(cfg)
    if not script.is_file():
        raise SystemExit(f"board power script not found: {script}")
    power_channel = str(cfg.get("power_channel", 4))
    print(f"[power] {action} port={power_port} channel={power_channel}", flush=True)
    subprocess.run(
        ["bash", str(script), action, power_port, power_channel],
        check=True,
        cwd=cfg["workspace"],
    )
    print(f"[power] {action} completed", flush=True)


def board_power_cycle(cfg: dict[str, Any]) -> None:
    board_power(cfg, "cycle")


def board_power_off(cfg: dict[str, Any]) -> None:
    board_power(cfg, "off")


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
        term.send(" ")
        time.sleep(0.2)
    raise TerminalTimeoutError(f"timed out waiting for U-Boot prompt (pattern={pattern!r})")


def save_inner_serial_log(cfg: dict[str, Any], content: str) -> None:
    if not content:
        return
    path = logs_dir(cfg) / "zone1_inner_serial.log"
    path.write_text(content, encoding="utf-8")


def stage_board_files(cfg: dict[str, Any]) -> None:
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
    env["TFTP_DIR"] = cfg["tftp_dir"]
    if cfg.get("zone1_dtb"):
        zone1_dtb = Path(cfg["zone1_dtb"])
        if not zone1_dtb.is_absolute():
            zone1_dtb = cfg["workspace"] / zone1_dtb
        env["ZONE1_DTB"] = str(zone1_dtb.resolve())
    env["COPY_HVISOR_BIN"] = "true" if cfg.get("copy_hvisor_bin", True) else "false"

    subprocess.run(["bash", str(script)], check=True, cwd=cfg["workspace"], env=env)


def deploy_chmod(cfg: dict[str, Any], term: Terminal) -> None:
    work_dir = cfg["zone1_work_dir"]
    boot_script = cfg["zone1_boot_script"]
    term.run(
        "deploy_chmod",
        f"chmod +x {work_dir}/{boot_script} {work_dir}/check_serial.sh 2>/dev/null || true",
        timeout=60.0,
    )


def deploy_board_pull(cfg: dict[str, Any], term: Terminal) -> int:
    """Board pulls staged zone1 files from the CI host via scp (zone0 serial console)."""
    board_ip = cfg.get("board_ip")
    board_iface = cfg.get("board_iface")
    if board_ip and board_iface:
        term.run(
            "deploy_set_ip",
            f"ifconfig {board_iface} {board_ip} netmask 255.255.255.0 up",
            timeout=30.0,
        )
        link_wait = float(cfg.get("deploy_link_wait", 0.0))
        if link_wait > 0:
            print(
                f"[deploy] waiting {link_wait}s for {board_iface} link on {board_ip}",
                flush=True,
            )
            time.sleep(link_wait)

    host_ip = cfg["network_host_ip"]
    ping_count = cfg["network_ping_count"]
    ping_retries = int(cfg.get("deploy_ping_retries", 1))
    ping_output = ""
    for attempt in range(ping_retries):
        if attempt > 0:
            retry_wait = float(cfg.get("deploy_link_wait", 5.0)) or 5.0
            print(f"[deploy] ping retry {attempt + 1}/{ping_retries} after {retry_wait}s", flush=True)
            time.sleep(retry_wait)
        _, ping_output = term.run(
            "deploy_ping",
            f"ping -c {ping_count} -W 5 {host_ip}",
            timeout=30.0 + ping_count * 5.0,
        )
        if ping_success(ping_output):
            break
    save_lspci_artifacts(cfg, "deploy_ping.log", f"=== ping {host_ip} ===\n{ping_output}\n")

    if not ping_success(ping_output):
        raise TerminalCommandError(f"ping {host_ip} failed")

    print(f"[deploy] ping {host_ip} ok, staging files on host", flush=True)
    stage_board_files(cfg)
    time.sleep(3.0)

    board_key = install_board_ssh_key(cfg, term)
    identity = board_scp_identity(cfg, board_key)

    host_user = cfg["network_host_user"]
    staging_dir = cfg["network_staging_dir"]
    work_dir = cfg["zone1_work_dir"]
    term.run("deploy_scp_host", f"CI_H={host_ip}", timeout=15.0)
    term.run("deploy_scp_user", f"CI_U={host_user}", timeout=15.0)
    term.run("deploy_scp_dir", f"CI_D={staging_dir}", timeout=15.0)
    scp_cmd = (
        f"scp {identity}-o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null "
        f"$CI_U@$CI_H:$CI_D/* {work_dir}/"
    )
    print(f"[deploy] pulling staged files from {host_user}@{host_ip}:{staging_dir}", flush=True)
    pull_rc, _ = term.run("deploy_scp_pull", scp_cmd, timeout=900.0)
    if pull_rc != 0:
        raise TerminalCommandError(f"board scp pull failed with rc={pull_rc}")

    deploy_chmod(cfg, term)
    print("[deploy] board_pull completed", flush=True)
    return 0


def deploy(cfg: dict[str, Any], term: Terminal | None) -> int:
    print("————————————————\ncase: deploy\n————————————————\n", flush=True)
    if cfg["mode"] != "board":
        print("[deploy] skipped (not board mode)", flush=True)
        return 0

    if term is None:
        raise SystemExit("terminal backend is required (run zone0_start first)")
    return deploy_board_pull(cfg, term)


def zone0_ready_pattern(cfg: dict[str, Any]) -> str:
    custom = str(cfg.get("zone0_ready_pattern", "")).strip()
    if custom:
        return custom
    return ZONE0_READY_PATTERN


def uboot_commands(cfg: dict[str, Any]) -> list[str]:
    raw = cfg.get("uboot_cmds")
    if isinstance(raw, list):
        cmds = [str(item).strip() for item in raw if str(item).strip()]
        if cmds:
            return cmds
    cmd = str(cfg.get("uboot_cmd", "")).strip()
    return [cmd] if cmd else []


def send_board_uboot_commands(cfg: dict[str, Any], term: Terminal) -> None:
    cmds = uboot_commands(cfg)
    if not cmds:
        return
    uboot_ready = cfg.get("uboot_ready_pattern", "") or r"=>"
    initial_timeout = float(cfg.get("uboot_prompt_timeout", 20.0))
    step_timeout = float(cfg.get("uboot_step_timeout", 0.0))
    if step_timeout <= 0:
        step_timeout = max(initial_timeout, 60.0)
    board_wait_uboot_prompt(term, uboot_ready, timeout=initial_timeout)
    for index, cmd in enumerate(cmds):
        if index > 0:
            time.sleep(0.3)
            board_wait_uboot_prompt(term, uboot_ready, timeout=step_timeout)
            time.sleep(0.3)
        term.send(cmd)


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
        cmds = uboot_commands(cfg)
        if cmds:
            uboot_ready = cfg.get("uboot_ready_pattern", "") or r"*=>"
            if not qemu_term.wait_pattern(uboot_ready, timeout=10.0):
                raise TerminalTimeoutError("timed out waiting for U-Boot prompt")
            for cmd in cmds:
                qemu_term.send(cmd)
        if not qemu_term.wait_pattern(zone0_ready_pattern(cfg), timeout=180.0):
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

        send_board_uboot_commands(cfg, board_term)
        zone0_timeout = float(cfg.get("zone0_shell_timeout", 180.0))
        if not board_term.wait_pattern(zone0_ready_pattern(cfg), timeout=zone0_timeout):
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


def install_board_ssh_key(cfg: dict[str, Any], term: Terminal) -> str | None:
    """Install a host-side private key on the board for scp pull. Returns key path on board."""
    key_path = cfg.get("deploy_board_ssh_key", "").strip()
    if not key_path:
        return None
    host_key = Path(key_path).expanduser()
    if not host_key.is_file():
        raise SystemExit(f"deploy board_ssh_key not found: {host_key}")

    board_key = f"/root/.ssh/{host_key.name}"
    key_b64 = base64.b64encode(host_key.read_bytes()).decode("ascii")
    cmd = (
        "mkdir -p /root/.ssh && chmod 700 /root/.ssh && "
        f"echo {key_b64} | base64 -d > {board_key} && chmod 600 {board_key}"
    )
    term.run("deploy_install_ssh_key", cmd, timeout=30.0)
    print(f"[deploy] installed board ssh key at {board_key}", flush=True)
    return board_key


def board_scp_identity(cfg: dict[str, Any], board_key: str | None) -> str:
    if board_key:
        return f"-i {board_key} "
    return ""


def zone1_start(cfg: dict[str, Any], term: Terminal | None) -> int:
    print("————————————————\ncase: zone1_start\n————————————————\n", flush=True)
    if term is None:
        raise SystemExit("terminal backend is required")

    work_dir = cfg["zone1_work_dir"]
    boot_script = cfg["zone1_boot_script"]
    inner_log = "/tmp/zone1_inner.log"
    _, _ = term.run("zone1_cd", f"cd {work_dir}", timeout=15.0)
    _, _ = term.run("zone1_ls", "ls", timeout=15.0)
    _, _ = term.run(
        "zone1_chmod",
        f"chmod +x {boot_script} hvisor check_serial.sh 2>/dev/null || true",
        timeout=30.0,
    )
    _, _ = term.run("zone1_cat_boot", f"cat {boot_script}", timeout=15.0)
    boot_rc, _ = term.run("zone1_boot", f"./{boot_script}", timeout=120.0)
    _, _ = term.run("zone1_list", "./hvisor zone list", timeout=15.0)

    start_wait = float(cfg.get("zone1_start_wait", 0.0))
    if start_wait > 0:
        print(f"[zone1] waiting {start_wait}s for zone1 console", flush=True)
        time.sleep(start_wait)

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
        raise TerminalCommandError(f"command failed with rc={boot_rc}: ./{boot_script}")
    if check_rc != 0:
        raise TerminalCommandError(
            f"command failed with rc={check_rc}: check_serial.sh (no console prompt)"
        )
    print("zone1_started successfully", flush=True)
    return 0


CASE_HANDLERS: dict[str, CaseFunc] = {
    "deploy": deploy,
    "zone0_start": zone0_start,
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
    deploy_cfg = tests.get("deploy") or tests.get("network") or {}
    if not isinstance(deploy_cfg, dict):
        deploy_cfg = {}

    def cfg_str(key: str, default: str = "") -> str:
        raw = deploy_cfg.get(key, tests.get(key, default))
        return str(raw).strip() if raw is not None else default

    deploy_method = str(deploy_cfg.get("method", "")).strip()
    if deploy_method and deploy_method != "board_pull":
        raise SystemExit(f"unsupported deploy method '{deploy_method}', only board_pull is supported")
    board_ip = cfg_str("board_ip")
    copy_hvisor_bin = deploy_cfg.get("copy_hvisor_bin", True)
    if isinstance(copy_hvisor_bin, str):
        copy_hvisor_bin = copy_hvisor_bin.lower() not in ("0", "false", "no")
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
        "power_channel": int(tests.get("power_channel", 4)),
        "baudrate": int(tests.get("baudrate", 1500000)),
        "uboot_cmd": str(tests.get("uboot_cmd", "")).strip(),
        "uboot_cmds": tests.get("uboot_cmds") or [],
        "uboot_ready_pattern": str(tests.get("uboot_ready_pattern", "")).strip(),
        "uboot_prompt_timeout": float(tests.get("uboot_prompt_timeout", 20.0)),
        "uboot_step_timeout": float(tests.get("uboot_step_timeout", 0.0)),
        "zone0_ready_pattern": str(tests.get("zone0_ready_pattern", "")).strip(),
        "zone0_shell_timeout": float(tests.get("zone0_shell_timeout", 300.0 if mode == "board" else 180.0)),
        "tftp_dir": str(tests.get("tftp_dir", "/home/light/tftp")).strip(),
        "lspci": tests.get("lspci") or {},
        "board_ip": board_ip,
        "board_iface": cfg_str("board_iface", "eth0"),
        "zone1_work_dir": cfg_str("zone1_work_dir", "/root"),
        "zone1_boot_script": cfg_str("zone1_boot_script", "boot_zone1.sh"),
        "network_host_ip": cfg_str("host_ip", "192.168.1.181"),
        "network_host_user": cfg_str("host_user", "light"),
        "network_staging_dir": cfg_str("staging_dir", "/home/light/tftp/ci_deploy"),
        "network_ping_count": int(deploy_cfg.get("ping_count", tests.get("ping_count", 3))),
        "deploy_link_wait": float(deploy_cfg.get("link_wait", 0.0)),
        "deploy_ping_retries": int(deploy_cfg.get("ping_retries", 1)),
        "deploy_board_ssh_key": cfg_str(
            "board_ssh_key", "/home/light/.ssh/hvisor_board_pull"
        ),
        "zone1_dtb": cfg_str("zone1_dtb"),
        "copy_hvisor_bin": bool(copy_hvisor_bin),
        "zone1_start_wait": float(tests.get("zone1_start_wait", 30.0)),
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

            standalone = case_name == "zone0_start"
            if standalone:
                try:
                    rc = case_fn(cfg, None)
                except (TerminalTimeoutError, TerminalCommandError) as exc:
                    print(f"[ci_runner] terminal command failed in case '{case_name}': {exc}", flush=True)
                    return 1
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
        if cfg.get("mode") == "board":
            board_power_off(cfg)


if __name__ == "__main__":
    raise SystemExit(main())
