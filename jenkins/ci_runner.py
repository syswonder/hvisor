#!/usr/bin/env python3
from __future__ import annotations

import argparse
import os
import re
import shlex
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
ZONE0_READY_PATTERN = r"root@[^\r\n]*#\s?|(?:\r?\n)#\s*(?:\r?\n|$)"
ZONE1_INNER_PROMPT_TIMEOUT = 60.0
ZONE1_INNER_LOG_FETCH_TIMEOUT = 30.0
DEPLOY_GUNZIP_ARTIFACTS = {"hvisor.gz": "hvisor"}
DEPLOY_LARGE_PULL_BYTES = 512 * 1024
DEPLOY_BOARD_TMP_DIR = "/tmp/ci_pull"
DEPLOY_BOARD_TMP_FILE = f"{DEPLOY_BOARD_TMP_DIR}/f"
DEPLOY_SPLIT_PART_RE = re.compile(r"^(.+)\.part\.[a-z]{2}$")
BOARD_HOME = "~"
BOARD_PUBKEY_LINE = re.compile(r"^ssh-(?:ed25519|rsa)\s+\S+")
BOARD_ROOT_SHELL_PATTERN = r"root@[^\r\n]*:\/#\s?|(?:\r?\n)#\s?(?:\r?\n|$)"
BOARD_SU_PASSWORD_PROMPT = r"Password:|password:"


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


def board_wait_uboot_prompt(
    term: Terminal,
    pattern: str,
    timeout: float,
    *,
    after_offset: int | None = None,
    autoboot_window: float = 0.0,
) -> None:
    """Wake the console and wait for a U-Boot prompt."""
    offset = term.offset() if after_offset is None else after_offset
    if autoboot_window > 0 and after_offset is None:
        deadline = time.monotonic() + autoboot_window
        while time.monotonic() < deadline:
            if term.wait_pattern(pattern, timeout=0.2, from_offset=offset):
                return
            term.send(" ")
            time.sleep(0.1)
    elif after_offset is None:
        term.send("")
        time.sleep(0.2)
        term.send("")
        time.sleep(0.2)
    if not term.wait_pattern(pattern, timeout=timeout, from_offset=offset):
        raise TerminalTimeoutError(f"timed out waiting for U-Boot prompt (pattern={pattern!r})")


def save_inner_serial_log(cfg: dict[str, Any], content: str) -> None:
    if not content:
        return
    path = logs_dir(cfg) / "zone1_inner_serial.log"
    path.write_text(content, encoding="utf-8")


def stage_board_files(cfg: dict[str, Any]) -> None:
    script = cfg["workspace"] / "jenkins" / "board_stage.sh"
    if not script.is_file():
        raise SystemExit(f"board stage script not found: {script}")

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


def save_case_log(cfg: dict[str, Any], name: str, content: str) -> None:
    path = logs_dir(cfg) / name
    path.write_text(content, encoding="utf-8")
    print(f"[ci_runner] saved {path}", flush=True)


def netmask_prefix(netmask: str) -> int:
    try:
        return sum(bin(int(part)).count("1") for part in netmask.split("."))
    except (ValueError, AttributeError):
        return 24


def board_sudo(cmd: str) -> str:
    """Wrap a board shell command to run with sudo -E (preserve HOME etc.)."""
    cmd = cmd.strip()
    if cmd.startswith("sudo "):
        return cmd
    if cmd.startswith("export "):
        return cmd
    return f"sudo -E sh -c {shlex.quote(cmd)}"


def board_priv(cfg: dict[str, Any], cmd: str) -> str:
    """Run privileged board commands; skip sudo when the shell is already root."""
    if cfg.get("board_is_root") or cfg.get("mode") == "qemu":
        return cmd.strip()
    return board_sudo(cmd)


def board_path(work_dir: str, *parts: str) -> str:
    """Build a shell path; ~ is left for the board shell to expand."""
    rel = "/".join(parts)
    if work_dir == BOARD_HOME:
        return f"{BOARD_HOME}/{rel}" if rel else BOARD_HOME
    return f"{work_dir}/{rel}" if rel else work_dir


def board_ssh_host(cfg: dict[str, Any]) -> str:
    return f"{cfg['network_host_user']}@{cfg['network_host_ip']}"


def _setup_board_interface(cfg: dict[str, Any], term: Terminal) -> None:
    """Assign a static IP and cycle the interface."""
    board_ip = str(cfg.get("board_ip", "")).strip()
    board_iface = str(cfg.get("board_iface", "")).strip()
    if not board_ip or not board_iface:
        return

    netmask = str(cfg.get("board_netmask", "255.255.255.0")).strip() or "255.255.255.0"
    prefix = netmask_prefix(netmask)
    term.run(
        "deploy_set_ip",
        board_priv(
            cfg,
            f"ip addr flush dev {board_iface} 2>/dev/null || true; "
            f"ip addr add {board_ip}/{prefix} dev {board_iface}",
        ),
        timeout=30.0,
    )
    term.run(
        "deploy_iface_down",
        board_priv(cfg, f"ip link set {board_iface} down"),
        timeout=15.0,
    )
    term.run(
        "deploy_iface_up",
        board_priv(cfg, f"ip link set {board_iface} up"),
        timeout=15.0,
    )

    link_wait = float(cfg.get("deploy_link_wait", 0.0))
    if link_wait > 0:
        print(
            f"[deploy] waiting {link_wait}s for {board_iface} link on {board_ip}",
            flush=True,
        )
        time.sleep(link_wait)
    time.sleep(2.0)


def _board_ping_host(cfg: dict[str, Any], term: Terminal) -> None:
    """Verify layer-3 reachability to the CI host before SSH setup."""
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
            board_priv(
                cfg,
                f"ping -c {ping_count} -W 5 {host_ip} > /tmp/ci_ping.log 2>&1; cat /tmp/ci_ping.log",
            ),
            timeout=30.0 + ping_count * 5.0,
            wake_interval=2.0,
        )
        if ping_success(ping_output):
            break
    save_case_log(cfg, "deploy_ping.log", f"=== ping {host_ip} ===\n{ping_output}\n")
    if not ping_success(ping_output):
        raise TerminalCommandError(f"ping {host_ip} failed")
    print(f"[deploy] ping {host_ip} ok", flush=True)


def ensure_board_net(cfg: dict[str, Any], term: Terminal) -> None:
    """Configure network, verify ping, and probe SSH to the CI host."""
    prepare_board_shell(cfg, term)
    _setup_board_interface(cfg, term)
    _board_ping_host(cfg, term)
    _board_ssh_probe(cfg, term)


def board_pubkey_from_output(output: str) -> str | None:
    for line in output.splitlines():
        candidate = line.strip()
        if BOARD_PUBKEY_LINE.match(candidate):
            return candidate
    return None


def _export_board_ssh_env(cfg: dict[str, Any], term: Terminal) -> None:
    if cfg.get("_board_ssh_env"):
        return
    connect = int(cfg.get("deploy_ssh_connect_timeout", 30))
    term.run(
        "board_ssh_env",
        (
            f"export CI_SSH_HOST={shlex.quote(board_ssh_host(cfg))} "
            f"CI_SSH_OPTS='-o BatchMode=yes -o ConnectTimeout={connect} "
            f"-o StrictHostKeyChecking=no'"
        ),
        timeout=15.0,
    )
    cfg["_board_ssh_env"] = True


def _board_ssh_probe(cfg: dict[str, Any], term: Terminal) -> None:
    _export_board_ssh_env(cfg, term)
    host_user = cfg["network_host_user"]
    host_ip = cfg["network_host_ip"]
    connect_timeout = int(cfg.get("deploy_ssh_connect_timeout", 30))
    board_limit = connect_timeout + 15
    probe_timeout = float(board_limit + 30)
    probe_rc, probe_out = term.run(
        "deploy_ssh_probe",
        board_priv(cfg, f"timeout {board_limit} ssh $CI_SSH_OPTS $CI_SSH_HOST true </dev/null"),
        timeout=probe_timeout,
    )
    if probe_rc == 0:
        print("[deploy] ssh to CI host ok", flush=True)
        return

    pub_rc, pub_out = term.run(
        "board_ssh_pubkey",
        board_priv(
            cfg,
            "cat ~/.ssh/id_ed25519.pub 2>/dev/null || cat ~/.ssh/id_rsa.pub 2>/dev/null || true",
        ),
        timeout=15.0,
    )
    pubkey = board_pubkey_from_output(pub_out) if pub_rc == 0 else None
    if not pubkey:
        raise TerminalCommandError(
            f"ssh probe to {host_user}@{host_ip} failed and board public key not found\n"
            f"{probe_out.strip()}"
        )

    auth_keys = f"/home/{host_user}/.ssh/authorized_keys"
    raise TerminalCommandError(
        f"ssh probe to {host_user}@{host_ip} failed; add this board public key to "
        f"{auth_keys} on the CI host:\n{pubkey}"
    )


def boot_board_zone0_shell(cfg: dict[str, Any], term: Terminal, *, power_cycle: bool = True) -> None:
    """Power-cycle (optional), run U-Boot commands, and prepare the zone0 shell."""
    if power_cycle:
        board_power_cycle(cfg)
    send_board_uboot_commands(cfg, term)
    timeout = float(cfg.get("zone0_shell_timeout", 180.0))
    if not term.wait_pattern(zone0_ready_pattern(cfg), timeout=timeout):
        raise TerminalTimeoutError("timed out waiting for zone0 shell prompt")
    prepare_board_shell(cfg, term)


def resolve_staging_dir(raw: str, arch: str, board: str) -> str:
    """Per-board staging path so parallel matrix cells do not clobber each other."""
    base = (raw or "/home/light/ci_deploy").rstrip("/")
    suffix = f"{arch}__{board.replace('/', '__')}"
    if base.endswith(suffix):
        return base
    return f"{base}/{suffix}"


def deploy_board_staging_alias(cfg: dict[str, Any]) -> str:
    """Symlink under /tmp so board scp commands fit the serial line limit."""
    alias = Path("/tmp/s") / cfg["board"].replace("/", "_")
    target = Path(cfg["network_staging_dir"])
    alias.parent.mkdir(parents=True, exist_ok=True)
    if alias.is_symlink() or alias.exists():
        alias.unlink()
    alias.symlink_to(target, target_is_directory=True)
    return str(alias)


def deploy_ensure_work_dir(cfg: dict[str, Any], term: Terminal) -> str:
    """Create zone1 work dir; fall back to ~ when configured dir is not writable."""
    preferred = cfg["zone1_work_dir"]
    candidates = [preferred]
    if preferred != BOARD_HOME:
        candidates.append(BOARD_HOME)

    board_term_run(
        cfg, term, "deploy_tmpdir", f"mkdir -p {DEPLOY_BOARD_TMP_DIR}", 15.0, retries=0
    )

    last_out = ""
    for path in candidates:
        probe = board_path(path, ".ci_probe")
        cmd = f"mkdir -p {board_path(path)} && touch {probe} && rm -f {probe}"
        rc, last_out = board_term_run(cfg, term, "deploy_mkdir", cmd, 30.0, retries=1)
        if rc == 0:
            if path != preferred:
                print(f"[deploy] warning: {preferred} unavailable, using {BOARD_HOME}", flush=True)
            else:
                print(f"[deploy] work dir ready: {path}", flush=True)
            cfg["zone1_work_dir"] = path
            return path
    raise TerminalCommandError(f"board work dir setup failed for {candidates!r}: {last_out.strip()}")


def parse_staging(staging_path: Path) -> tuple[list[str], dict[str, list[str]], list[str]]:
    regular: list[str] = []
    split_groups: dict[str, list[str]] = {}

    for entry in staging_path.iterdir():
        if not entry.is_file():
            continue
        name = entry.name
        match = DEPLOY_SPLIT_PART_RE.match(name)
        if match:
            split_groups.setdefault(match.group(1), []).append(name)
            continue
        regular.append(name)

    for base in split_groups:
        split_groups[base] = sorted(split_groups[base])
    parts = [part for part_names in split_groups.values() for part in part_names]
    pull_files = sorted(
        regular + parts,
        key=lambda name: staging_path.joinpath(name).stat().st_size,
    )
    return regular, split_groups, pull_files


def deploy_fail(cfg: dict[str, Any], all_logs: list[str], section: str, output: str) -> None:
    all_logs.append(f"=== {section} ===\n{output}\n")
    save_case_log(cfg, "deploy_scp.log", "".join(all_logs))


def deploy_board_cmd(
    cfg: dict[str, Any],
    term: Terminal,
    all_logs: list[str],
    case_id: str,
    command: str,
    timeout: float,
    error: str,
    *,
    retries: int = 1,
) -> str:
    try:
        rc, out = board_term_run(cfg, term, case_id, command, timeout, retries=retries)
    except TerminalTimeoutError as exc:
        deploy_fail(cfg, all_logs, f"{case_id} (timed out)", exc.partial_output or "")
        raise
    if rc != 0:
        deploy_fail(cfg, all_logs, case_id, out)
        raise TerminalCommandError(f"{error}: {out.strip()}")
    all_logs.append(f"=== {case_id} ===\n{out}\n")
    return out


def deploy_scp_to_tmp(
    cfg: dict[str, Any],
    term: Terminal,
    all_logs: list[str],
    name: str,
    tmp_path: str,
    file_size: int,
    file_timeout: float,
) -> None:
    """Scp one file to tmp_path; recover when serial loses the result marker."""
    scp_cmd = (
        f"scp -q $CI_SSH_OPTS $CI_SSH_HOST:$CI_D/{name} {tmp_path} </dev/null 2>/dev/null"
    )
    case_id = f"deploy_scp_{name}"
    try:
        deploy_board_cmd(
            cfg, term, all_logs, case_id, scp_cmd, file_timeout,
            f"board scp failed for {name!r}", retries=0,
        )
    except TerminalTimeoutError:
        check = f"test $(wc -c < {tmp_path}) -eq {file_size}"
        rc, out = board_term_run(cfg, term, f"{case_id}_recover", check, 20.0, retries=0)
        if rc != 0:
            deploy_fail(cfg, all_logs, f"{case_id} (timed out)", out)
            raise
        all_logs.append(f"=== {case_id} (recovered) ===\n{out}\n")
        print(f"[deploy] scp recovered after serial timeout: {name}", flush=True)


def deploy_install_from_tmp(
    cfg: dict[str, Any],
    term: Terminal,
    all_logs: list[str],
    name: str,
    final_path: str,
    file_size: int,
) -> None:
    install_cmd = f"cp $CI_F {final_path} && rm -f $CI_F"
    case_id = f"deploy_install_{name}"
    try:
        deploy_board_cmd(
            cfg, term, all_logs, case_id, install_cmd, 30.0,
            f"board install failed for {name!r}", retries=0,
        )
    except TerminalTimeoutError:
        check = f"test $(wc -c < {final_path}) -eq {file_size}"
        rc, out = board_term_run(cfg, term, f"{case_id}_recover", check, 20.0, retries=0)
        if rc != 0:
            deploy_fail(cfg, all_logs, f"{case_id} (timed out)", out)
            raise
        all_logs.append(f"=== {case_id} (recovered) ===\n{out}\n")
        print(f"[deploy] install recovered after serial timeout: {name}", flush=True)


def deploy_board_pull_file(
    cfg: dict[str, Any],
    term: Terminal,
    staging_path: Path,
    staging_dir: str,
    work_dir: str,
    name: str,
    deadline: float,
    all_logs: list[str],
    *,
    index: int,
    total: int,
) -> None:
    remaining = deadline - time.monotonic()
    if remaining <= 0:
        save_case_log(cfg, "deploy_scp.log", "".join(all_logs))
        raise TerminalTimeoutError(
            f"deploy pull timed out before {name!r}",
            partial_output="".join(all_logs),
        )

    file_size = staging_path.joinpath(name).stat().st_size
    file_timeout = deploy_file_pull_timeout(file_size, remaining)
    tmp_path = DEPLOY_BOARD_TMP_FILE
    final_path = board_path(work_dir, name)
    print(
        f"[deploy] scp ({index}/{total}): {name} ({file_size} bytes, timeout={file_timeout:.0f}s)",
        flush=True,
    )
    deploy_scp_to_tmp(cfg, term, all_logs, name, tmp_path, file_size, file_timeout)
    deploy_install_from_tmp(cfg, term, all_logs, name, final_path, file_size)


def deploy_assemble_split_file(
    cfg: dict[str, Any],
    term: Terminal,
    staging_path: Path,
    work_dir: str,
    base: str,
    parts: list[str],
    all_logs: list[str],
) -> None:
    expected_size = sum(staging_path.joinpath(part).stat().st_size for part in parts)
    part_args = " ".join(board_path(work_dir, part) for part in parts)
    out_path = board_path(work_dir, base)
    cat_cmd = f"cat {part_args} > {out_path}"
    verify_cmd = f"test $(wc -c < {out_path}) -eq {expected_size}"
    print(f"[deploy] assemble split file {base} from {len(parts)} part(s)", flush=True)
    deploy_board_cmd(
        cfg, term, all_logs, f"deploy_cat_{base}", cat_cmd, 120.0,
        f"board assemble failed for {base!r}", retries=1,
    )
    deploy_board_cmd(
        cfg, term, all_logs, f"deploy_verify_{base}", verify_cmd, 30.0,
        f"board assemble verify failed for {base!r}: expected {expected_size} bytes",
        retries=1,
    )
    rm_parts = " ".join(board_path(work_dir, part) for part in parts)
    term.run(f"deploy_rm_parts_{base}", board_priv(cfg, f"rm -f {rm_parts}"), timeout=30.0)


def deploy_gunzip_artifact(
    cfg: dict[str, Any],
    term: Terminal,
    work_dir: str,
    gz_name: str,
    all_logs: list[str],
) -> None:
    raw_name = DEPLOY_GUNZIP_ARTIFACTS[gz_name]
    raw_path = Path(cfg["hvisor_tool_path"]) / "output" / raw_name
    raw_size = raw_path.stat().st_size
    gunzip_cmd = (
        f"gunzip -f {board_path(work_dir, gz_name)} && "
        f"test $(wc -c < {board_path(work_dir, raw_name)}) -eq {raw_size}"
    )
    print(f"[deploy] gunzip: {gz_name} -> {raw_name}", flush=True)
    deploy_board_cmd(
        cfg, term, all_logs, f"deploy_gunzip_{raw_name}", gunzip_cmd, 120.0,
        f"board gunzip failed for {gz_name!r}",
    )


def board_term_run(
    cfg: dict[str, Any],
    term: Terminal,
    case_id: str,
    command: str,
    timeout: float,
    *,
    retries: int = 1,
) -> tuple[int, str]:
    last_exc: TerminalTimeoutError | None = None
    for attempt in range(retries + 1):
        if attempt > 0:
            print(f"[deploy] retry {case_id} ({attempt}/{retries})", flush=True)
            time.sleep(3.0)
        run_id = case_id if attempt == 0 else f"{case_id}_retry{attempt}"
        try:
            return term.run(run_id, board_priv(cfg, command), timeout=timeout)
        except TerminalTimeoutError as exc:
            last_exc = exc
    assert last_exc is not None
    raise last_exc


def deploy_board_pull(cfg: dict[str, Any], term: Terminal) -> int:
    """Board pulls staged zone1 files from the CI host over scp."""
    ensure_board_net(cfg, term)

    host_ip = cfg["network_host_ip"]
    host_user = cfg["network_host_user"]

    print("[deploy] staging files on host", flush=True)
    stage_board_files(cfg)

    staging_dir = deploy_board_staging_alias(cfg)
    staging_path = Path(cfg["network_staging_dir"])
    if not staging_path.is_dir():
        raise TerminalCommandError(f"staging dir not found: {staging_path}")

    work_dir = deploy_ensure_work_dir(cfg, term)

    regular_files, split_groups, pull_files = parse_staging(staging_path)
    if not pull_files and not split_groups:
        raise TerminalCommandError(f"no staged files in {staging_path}")

    pull_timeout = float(cfg.get("deploy_pull_timeout", 600.0))
    deadline = time.monotonic() + pull_timeout
    all_logs: list[str] = []

    stale = sorted(set(pull_files) | set(split_groups) | set(DEPLOY_GUNZIP_ARTIFACTS.values()))
    print(f"[deploy] removing stale files in {work_dir}", flush=True)
    for name in stale:
        stale_path = board_path(work_dir, name)
        term.run(f"deploy_clean_{name}", board_priv(cfg, f"rm -f {stale_path}"), timeout=15.0)

    print(
        f"[deploy] pulling {len(pull_files)} file(s) via scp from {host_user}@{host_ip}:{staging_dir}",
        flush=True,
    )
    if split_groups:
        print(f"[deploy] split artifacts: {', '.join(sorted(split_groups))}", flush=True)

    board_term_run(
        cfg,
        term,
        "deploy_pull_env",
        f"export CI_D={shlex.quote(staging_dir)} CI_F={DEPLOY_BOARD_TMP_FILE}",
        30.0,
        retries=0,
    )

    for index, name in enumerate(pull_files, start=1):
        if index > 1:
            time.sleep(3.0)
        deploy_board_pull_file(
            cfg,
            term,
            staging_path,
            staging_dir,
            work_dir,
            name,
            deadline,
            all_logs,
            index=index,
            total=len(pull_files),
        )

    for base, parts in sorted(split_groups.items()):
        deploy_assemble_split_file(cfg, term, staging_path, work_dir, base, parts, all_logs)

    for gz_name in DEPLOY_GUNZIP_ARTIFACTS:
        if gz_name in regular_files or gz_name in split_groups:
            deploy_gunzip_artifact(cfg, term, work_dir, gz_name, all_logs)

    save_case_log(cfg, "deploy_scp.log", "".join(all_logs))

    boot_script = cfg["zone1_boot_script"]
    term.run(
        "deploy_chmod",
        board_priv(
            cfg,
            f"chmod +x {board_path(work_dir, boot_script)} "
            f"{board_path(work_dir, 'hvisor')} {board_path(work_dir, 'check_serial.sh')} 2>/dev/null || true",
        ),
        timeout=60.0,
    )
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


def board_shell_ready_pattern(cfg: dict[str, Any]) -> str:
    custom = str(cfg.get("board_shell_ready_pattern", "")).strip()
    if custom:
        return custom
    board_user = str(cfg.get("board_user", "root")).strip() or "root"
    if board_user == "root":
        return BOARD_ROOT_SHELL_PATTERN
    return rf"{re.escape(board_user)}@[^\r\n]*[$#]\s?"


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

    autoboot_window = float(cfg.get("uboot_autoboot_window", 0.0))
    board_wait_uboot_prompt(
        term,
        uboot_ready,
        timeout=initial_timeout,
        autoboot_window=autoboot_window,
    )
    for index, cmd in enumerate(cmds):
        send_offset = term.offset()
        term.send(cmd)
        if index < len(cmds) - 1:
            board_wait_uboot_prompt(
                term,
                uboot_ready,
                timeout=step_timeout,
                after_offset=send_offset,
            )


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
        cfg["board_is_root"] = True
        cfg["_board_shell_ready"] = True
        return 0
    if cfg["mode"] == "board":
        log_path = logs_dir(cfg) / "zone0_console.log"
        log_path.write_text("", encoding="utf-8")

        board_term = build_terminal(cfg, log_path)
        board_term.open()
        cfg["_board_term"] = board_term
        boot_board_zone0_shell(cfg, board_term, power_cycle=True)
        return 0
    return 0


def lspci_expected_cfg(cfg: dict[str, Any]) -> dict[str, Any]:
    raw = cfg.get("lspci") or {}
    if not isinstance(raw, dict):
        raw = {}
    expected_bdfs = [str(x) for x in raw.get("expected_bdfs") or []]
    min_count = int(raw.get("min_count", len(expected_bdfs) or 1))
    return {"expected_bdfs": expected_bdfs, "min_count": min_count}


def lspci(cfg: dict[str, Any], term: Terminal | None) -> int:
    print("————————————————\ncase: lspci\n————————————————\n", flush=True)
    if term is None:
        raise SystemExit("terminal backend is required (run zone0_start first)")

    expected = lspci_expected_cfg(cfg)

    _, output = term.run(
        "lspci",
        board_priv(cfg, "timeout 20 lspci -D > /tmp/ci_lspci.log 2>&1; cat /tmp/ci_lspci.log"),
        timeout=60.0,
    )
    save_case_log(cfg, "lspci.log", f"=== lspci -D ===\n{output}\n")

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
    if re.search(r"0%\s*(?:packet\s*)?loss\b", output, re.I):
        return True
    if re.search(r"0%\s*包丢失", output):
        return True
    match = re.search(r"(\d+)\s*packets?\s*received", output, re.I)
    if match and int(match.group(1)) > 0:
        return True
    match = re.search(r"已接收\s*(\d+)\s*个包", output)
    return bool(match and int(match.group(1)) > 0)


def board_login_user(term: Terminal) -> str:
    _, out = term.run("board_id", "id -un", timeout=15.0)
    for line in out.splitlines():
        candidate = line.strip()
        if candidate and "__R__" not in candidate and re.fullmatch(r"[\w.-]+", candidate):
            return candidate
    raise TerminalCommandError(f"failed to detect board login user: {out.strip()!r}")


def ensure_board_shell_user(cfg: dict[str, Any], term: Terminal) -> None:
    """Switch shell user when board_shell_su_cmd is set in ci.yaml."""
    su_cmd = str(cfg.get("board_shell_su_cmd", "")).strip()
    if not su_cmd:
        return
    target = str(cfg.get("board_user", "root")).strip() or "root"
    if board_login_user(term) == target:
        return

    shell_prompt = board_shell_ready_pattern(cfg)
    password = str(cfg.get("board_su_password", ""))
    print(f"[deploy] {su_cmd}", flush=True)
    offset = term.offset()
    term.send(su_cmd)
    deadline = time.monotonic() + 20.0
    sent_password = False
    while time.monotonic() < deadline:
        chunk = term.tail_since(offset)
        if not sent_password and re.search(BOARD_SU_PASSWORD_PROMPT, chunk):
            term.send(password)
            sent_password = True
        if re.search(shell_prompt, chunk):
            break
        time.sleep(0.1)
    else:
        raise TerminalCommandError(f"timed out waiting for shell after {su_cmd!r}")

    # Serial su may print job-control errors; an extra enter wakes the ready shell.
    time.sleep(0.3)
    term.send("")
    time.sleep(0.3)
    if board_login_user(term) != target:
        raise TerminalCommandError(f"failed to switch shell user to {target}")


def prepare_board_shell(cfg: dict[str, Any], term: Terminal) -> None:
    """Enter board shell once: optional su, detect root/user, export HOME."""
    if cfg.get("_board_shell_ready"):
        return

    ensure_board_shell_user(cfg, term)
    user = board_login_user(term)
    cfg["board_is_root"] = user == "root"
    term.run("board_home_export", "export HOME=$(cd ~ && pwd)", timeout=15.0)
    cfg["_board_shell_ready"] = True
    print(f"[board] shell ready: user={user} home={BOARD_HOME}", flush=True)


def deploy_file_pull_timeout(file_size: int, remaining: float) -> float:
    """Per-file timeout from size (8 KiB/s floor) capped by the deploy deadline."""
    min_bps = 8192.0
    needed = max(90.0, file_size / min_bps + 45.0)
    return min(remaining, needed)


def zone1_start(cfg: dict[str, Any], term: Terminal | None) -> int:
    print("————————————————\ncase: zone1_start\n————————————————\n", flush=True)
    if term is None:
        raise SystemExit("terminal backend is required")

    work_dir = board_path(cfg["zone1_work_dir"])
    boot_script = cfg["zone1_boot_script"]
    inner_log = "/tmp/zone1_inner.log"
    _, _ = term.run("zone1_ls", f"cd {work_dir} && ls", timeout=15.0)
    boot_rc, _ = term.run("zone1_boot", f"cd {work_dir} && ./{boot_script}", timeout=120.0)
    if str(cfg.get("board_user", "root")).strip() not in ("", "root"):
        zone_list_cmd = f"cd {work_dir} && sudo ./hvisor zone list"
    else:
        zone_list_cmd = f"cd {work_dir} && ./hvisor zone list"
    _, _ = term.run("zone1_list", zone_list_cmd, timeout=15.0)

    start_wait = float(cfg.get("zone1_start_wait", 0.0))
    if start_wait > 0:
        print(f"[zone1] waiting {start_wait}s for zone1 console", flush=True)
        time.sleep(start_wait)

    max_pts = find_zone1_pts(term)

    check_rc, _ = term.run(
        "zone1_serial_check",
        (
            f"cd {work_dir} && ./check_serial.sh /dev/pts/{max_pts} {inner_log} "
            f"{int(ZONE1_INNER_PROMPT_TIMEOUT)}"
        ),
        timeout=ZONE1_INNER_PROMPT_TIMEOUT + 30.0,
    )
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

    board_ip = cfg_str("board_ip")
    board_user = cfg_str("board_user", "root") or "root"
    default_work_dir = "/root" if board_user == "root" else f"/home/{board_user}"
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
        "uboot_autoboot_window": float(tests.get("uboot_autoboot_window", 0.0)),
        "zone0_ready_pattern": str(tests.get("zone0_ready_pattern", "")).strip(),
        "board_shell_ready_pattern": str(tests.get("board_shell_ready_pattern", "")).strip(),
        "board_shell_su_cmd": str(tests.get("board_shell_su_cmd", "")).strip(),
        "zone0_shell_timeout": float(tests.get("zone0_shell_timeout", 300.0 if mode == "board" else 180.0)),
        "tftp_dir": str(tests.get("tftp_dir", "/home/light/tftp")).strip(),
        "lspci": tests.get("lspci") or {},
        "board_ip": board_ip,
        "board_user": board_user,
        "board_iface": cfg_str("board_iface", "eth0"),
        "board_netmask": cfg_str("netmask", "255.255.255.0"),
        "board_su_password": cfg_str("su_password", ""),
        "zone1_work_dir": cfg_str("zone1_work_dir", default_work_dir),
        "zone1_boot_script": cfg_str("zone1_boot_script", "boot_zone1.sh"),
        "network_host_ip": cfg_str("host_ip", "192.168.1.181"),
        "network_host_user": cfg_str("host_user", "light"),
        "network_staging_dir": resolve_staging_dir(
            cfg_str("staging_dir", "/home/light/ci_deploy"),
            arch,
            board,
        ),
        "network_ping_count": int(deploy_cfg.get("ping_count", tests.get("ping_count", 3))),
        "deploy_link_wait": float(deploy_cfg.get("link_wait", 0.0)),
        "deploy_ping_retries": int(deploy_cfg.get("ping_retries", 1)),
        "deploy_ssh_connect_timeout": int(deploy_cfg.get("ssh_connect_timeout", 30)),
        "deploy_pull_timeout": float(deploy_cfg.get("pull_timeout", 600.0)),
        "zone1_dtb": cfg_str("zone1_dtb"),
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
