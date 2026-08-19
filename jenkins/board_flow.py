#!/usr/bin/env python3
"""Board CI flow helpers (zone0, login, network_and_trans, zone1)."""

from __future__ import annotations

import os
import re
import shlex
import subprocess
import time
import uuid
from collections.abc import Callable
from pathlib import Path
from typing import Any

from terminal import Terminal, TerminalCommandError, TerminalTimeoutError

ZONE0_READY_PATTERN = r"root@[^\r\n]*#\s?|(?:\r?\n)#\s*(?:\r?\n|$)"
ZONE1_INNER_PROMPT_TIMEOUT = 60.0
ZONE1_PTS_PATTERN = r"/dev/pts/\d+"
GUNZIP_ARTIFACTS = {"hvisor.gz": "hvisor"}
SPLIT_PART_RE = re.compile(r"^(.+)\.part\.[a-z]{2}$")
BOARD_PUBKEY_LINE = re.compile(r"^ssh-(?:ed25519|rsa)\s+\S+")
BOARD_CMD_MARKER_RE = re.compile(r"__HV_M_(?P<run_id>[a-f0-9]{4}):(?P<rc>\d+)")
MAX_BOARD_CMD_LEN = 128
LOGIN_TOOLS_DEFAULT = [
    "ping",
    "ssh",
    "scp",
    "gzip",
    "gunzip",
    "cat",
    "ip",
    "test",
    "wc",
    "mkdir",
    "timeout",
]


def logs_dir(cfg: dict[str, Any]) -> Path:
    path = Path(cfg["workspace"]) / "logs"
    path.mkdir(parents=True, exist_ok=True)
    return path


def release_logs_ownership(cfg: dict[str, Any]) -> None:
    """Return logs/ to the invoking user when ci_runner ran under sudo."""
    if os.geteuid() != 0:
        return
    sudo_uid = os.environ.get("SUDO_UID")
    sudo_gid = os.environ.get("SUDO_GID")
    if not sudo_uid or not sudo_gid:
        return
    uid, gid = int(sudo_uid), int(sudo_gid)
    path = Path(cfg["workspace"]) / "logs"
    if not path.is_dir():
        return
    for root, dirs, files in os.walk(path, topdown=False):
        for name in files:
            os.chown(os.path.join(root, name), uid, gid)
        for name in dirs:
            os.chown(os.path.join(root, name), uid, gid)
    os.chown(path, uid, gid)


def save_case_log(cfg: dict[str, Any], name: str, content: str) -> None:
    path = logs_dir(cfg) / name
    path.write_text(content, encoding="utf-8")
    print(f"[board_flow] saved {path}", flush=True)


def retry_step(
    name: str,
    fn: Callable[[], None],
    retries: int = 2,
    delay: float = 3.0,
    *,
    on_retry: Callable[[], None] | None = None,
) -> None:
    last_exc: Exception | None = None
    for attempt in range(retries + 1):
        if attempt > 0:
            print(f"[board_flow] retry {name} ({attempt}/{retries})", flush=True)
            if on_retry is not None:
                on_retry()
            time.sleep(delay)
        try:
            fn()
            return
        except (TerminalTimeoutError, TerminalCommandError) as exc:
            last_exc = exc
    assert last_exc is not None
    raise last_exc


def shell_pattern(cfg: dict[str, Any]) -> str:
    custom = str(cfg.get("zone0_ready_pattern", "")).strip()
    return custom or ZONE0_READY_PATTERN


def board_wait_shell(term: Terminal, cfg: dict[str, Any], timeout: float = 30.0) -> None:
    # Send Enter so the shell prints a fresh prompt in new collector output.
    # wait_pattern only searches from the current offset onward.
    term.send("")
    if not term.wait_pattern(shell_pattern(cfg), timeout=timeout):
        raise TerminalTimeoutError("timed out waiting for shell prompt")


def board_run(
    term: Terminal,
    command: str,
    *,
    timeout: float = 30.0,
) -> tuple[int, str]:
    """Run a short shell command; completion is detected via wait_pattern on a marker."""
    if len(command) > MAX_BOARD_CMD_LEN:
        raise ValueError(
            f"board command too long ({len(command)} > {MAX_BOARD_CMD_LEN}): {command!r}"
        )
    run_id = uuid.uuid4().hex[:4]
    marker_needle = rf"__HV_M_{run_id}:\d+"
    offset = term.offset()
    term.send(f"{command}; echo __HV_M_{run_id}:$?")

    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        remaining = deadline - time.monotonic()
        if remaining <= 0:
            break
        if term.wait_pattern(marker_needle, timeout=min(0.2, remaining), from_offset=offset):
            chunk = term.tail_since(offset)
            matches = [m for m in BOARD_CMD_MARKER_RE.finditer(chunk) if m.group("run_id") == run_id]
            if matches:
                match = matches[-1]
                rc = int(match.group("rc"))
                return rc, chunk[: match.start()]

    raise TerminalTimeoutError(
        f"timed out waiting for board command marker __HV_M_{run_id}",
        partial_output=term.tail_since(offset),
    )


def board_run_check(
    term: Terminal,
    command: str,
    *,
    timeout: float = 30.0,
) -> str:
    rc, output = board_run(term, command, timeout=timeout)
    if rc != 0:
        raise TerminalCommandError(f"command failed with rc={rc}: {command}\n{output.strip()}")
    return output


def board_run_retry(
    term: Terminal,
    command: str,
    timeout: float,
    *,
    retries: int = 1,
    check_exit: bool = False,
) -> tuple[int, str]:
    last_exc: TerminalTimeoutError | None = None
    for attempt in range(retries + 1):
        if attempt > 0:
            time.sleep(3.0)
        try:
            rc, output = board_run(term, command, timeout=timeout)
            if check_exit and rc != 0:
                raise TerminalCommandError(
                    f"command failed with rc={rc}: {command}\n{output.strip()}"
                )
            return rc, output
        except TerminalTimeoutError as exc:
            last_exc = exc
    assert last_exc is not None
    raise last_exc


def board_power(cfg: dict[str, Any], action: str) -> None:
    power_port = str(cfg.get("power_serial", "")).strip()
    if not power_port:
        print(f"[power] skip {action}: power_serial is empty", flush=True)
        return
    script = cfg["workspace"] / "jenkins" / "board_power.sh"
    if not script.is_file():
        raise SystemExit(f"board power script not found: {script}")
    channel = str(cfg.get("power_channel", 4))
    print(f"[power] {action} port={power_port} channel={channel}", flush=True)
    subprocess.run(
        ["bash", str(script), action, power_port, channel],
        check=True,
        cwd=cfg["workspace"],
    )


def board_power_cycle(cfg: dict[str, Any]) -> None:
    board_power(cfg, "cycle")


def board_power_off(cfg: dict[str, Any]) -> None:
    board_power(cfg, "off")


def close_board_terminal(cfg: dict[str, Any]) -> None:
    term = cfg.get("_board_term")
    if term is not None:
        term.close()
        cfg["_board_term"] = None


def get_board_terminal(cfg: dict[str, Any]) -> Terminal | None:
    return cfg.get("_board_term")


def zone0_ready_pattern(cfg: dict[str, Any]) -> str:
    return shell_pattern(cfg)


def uboot_commands(cfg: dict[str, Any]) -> list[str]:
    raw = cfg.get("uboot_cmds")
    if isinstance(raw, list):
        cmds = [str(item).strip() for item in raw if str(item).strip()]
        if cmds:
            return cmds
    cmd = str(cfg.get("uboot_cmd", "")).strip()
    return [cmd] if cmd else []


def board_wait_uboot_prompt(
    term: Terminal,
    pattern: str,
    timeout: float,
    *,
    autoboot_window: float = 0.0,
) -> None:
    if autoboot_window > 0:
        deadline = time.monotonic() + autoboot_window
        while time.monotonic() < deadline:
            remaining = deadline - time.monotonic()
            if remaining <= 0:
                break
            if term.wait_pattern(pattern, timeout=min(0.2, remaining)):
                return
            term.send(" ")
    else:
        term.send("")
        term.send("")
    if not term.wait_pattern(pattern, timeout=timeout):
        raise TerminalTimeoutError(f"timed out waiting for U-Boot prompt (pattern={pattern!r})")


def send_board_uboot_commands(cfg: dict[str, Any], term: Terminal) -> None:
    cmds = uboot_commands(cfg)
    if not cmds:
        return
    uboot_ready = str(cfg.get("uboot_ready_pattern", "")).strip() or r"=>"
    initial_timeout = float(cfg.get("uboot_prompt_timeout", 60.0))
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
        term.send(cmd)
        if index < len(cmds) - 1:
            board_wait_uboot_prompt(term, uboot_ready, timeout=step_timeout)


def boot_board_zone0(cfg: dict[str, Any], term: Terminal) -> None:
    board_power_cycle(cfg)
    send_board_uboot_commands(cfg, term)
    timeout = float(cfg.get("zone0_shell_timeout", 180.0))
    if not term.wait_pattern(zone0_ready_pattern(cfg), timeout=timeout):
        raise TerminalTimeoutError("timed out waiting for zone0 shell prompt")


def board_login_user(term: Terminal, cfg: dict[str, Any]) -> str:
    _, out = board_run(term, "id -un", timeout=15.0)
    for line in out.splitlines():
        candidate = line.strip()
        if candidate and re.fullmatch(r"[\w.-]+", candidate):
            return candidate
    raise TerminalCommandError(f"failed to detect board login user: {out.strip()!r}")


def prepare_board_shell(cfg: dict[str, Any], term: Terminal) -> None:
    if cfg.get("_board_shell_ready"):
        return
    user = board_login_user(term, cfg)
    cfg["board_is_root"] = user == "root"
    board_run(term, "export HOME=$(cd ~ && pwd)", timeout=15.0)
    cfg["_board_shell_ready"] = True
    print(f"[board] shell ready: user={user}", flush=True)


def check_login_tools(cfg: dict[str, Any], term: Terminal) -> None:
    tools = cfg.get("login_tools") or LOGIN_TOOLS_DEFAULT
    missing: list[str] = []
    for tool in tools:
        name = str(tool)
        rc, _ = board_run(
            term,
            f"command -v {shlex.quote(name)} >/dev/null",
            timeout=15.0,
        )
        if rc != 0:
            missing.append(name)
    if missing:
        print(f"[login] WARNING: missing required tools: {missing}", flush=True)
        raise SystemExit(1)
    print(f"[login] all required tools available ({len(tools)})", flush=True)


def netmask_prefix(netmask: str) -> int:
    try:
        return sum(bin(int(part)).count("1") for part in netmask.split("."))
    except (ValueError, AttributeError):
        return 24


def board_ssh_host(cfg: dict[str, Any]) -> str:
    return f"{cfg['host_user']}@{cfg['host_ip']}"


def board_path(work_dir: str, *parts: str) -> str:
    rel = "/".join(parts)
    if work_dir == "~":
        return f"~/{rel}" if rel else "~"
    return f"{work_dir}/{rel}" if rel else work_dir


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


def board_pubkey_from_output(output: str) -> str | None:
    for line in output.splitlines():
        candidate = line.strip()
        if BOARD_PUBKEY_LINE.match(candidate):
            return candidate
    return None


def export_board_ssh_env(cfg: dict[str, Any], term: Terminal) -> None:
    if cfg.get("_board_ssh_env"):
        return
    connect = int(cfg.get("ssh_connect_timeout", 60))
    board_run(term, f"export CI_SSH_HOST={shlex.quote(board_ssh_host(cfg))}", timeout=15.0)
    board_run(term, 'export CI_SSH_OPTS="-o BatchMode=yes"', timeout=15.0)
    board_run(term, f'export CI_SSH_OPTS="$CI_SSH_OPTS -o ConnectTimeout={connect}"', timeout=15.0)
    board_run(
        term,
        'export CI_SSH_OPTS="$CI_SSH_OPTS -o StrictHostKeyChecking=no"',
        timeout=15.0,
    )
    cfg["_board_ssh_env"] = True


def setup_board_interface(cfg: dict[str, Any], term: Terminal) -> None:
    board_ip = str(cfg.get("board_ip", "")).strip()
    board_iface = str(cfg.get("board_iface", "")).strip()
    if not board_ip or not board_iface:
        return
    netmask = str(cfg.get("board_netmask", "255.255.255.0")).strip() or "255.255.255.0"
    prefix = netmask_prefix(netmask)
    board_run(term, f"ip addr flush dev {board_iface} 2>/dev/null || true", timeout=15.0)
    board_run(term, f"ip addr add {board_ip}/{prefix} dev {board_iface}", timeout=30.0)
    board_run(term, f"ip link set {board_iface} down", timeout=15.0)
    board_run(term, f"ip link set {board_iface} up", timeout=15.0)
    link_wait = float(cfg.get("link_wait", 0.0))
    if link_wait > 0:
        print(f"[network] waiting {link_wait}s for link", flush=True)
        time.sleep(link_wait)
    board_wait_shell(term, cfg, timeout=10.0)


def ping_host(cfg: dict[str, Any], term: Terminal) -> None:
    host_ip = cfg["host_ip"]
    ping_count = int(cfg.get("ping_count", 3))
    ping_retries = int(cfg.get("ping_retries", 1))
    ping_output = ""
    for attempt in range(ping_retries):
        if attempt > 0:
            retry_wait = float(cfg.get("link_wait", 5.0)) or 5.0
            print(f"[network] ping retry {attempt + 1}/{ping_retries}", flush=True)
            time.sleep(retry_wait)
        board_run(
            term,
            f"ping -c {ping_count} -W 5 {host_ip} >/tmp/ci_ping.log 2>&1",
            timeout=30.0 + ping_count * 5.0,
        )
        _, ping_output = board_run(term, "cat /tmp/ci_ping.log", timeout=15.0)
        if ping_success(ping_output):
            break
    save_case_log(cfg, "deploy_ping.log", f"=== ping {host_ip} ===\n{ping_output}\n")
    if not ping_success(ping_output):
        raise TerminalCommandError(f"ping {host_ip} failed")
    print(f"[network] ping {host_ip} ok", flush=True)


def probe_board_ssh(cfg: dict[str, Any], term: Terminal) -> None:
    export_board_ssh_env(cfg, term)
    host_user = cfg["host_user"]
    host_ip = cfg["host_ip"]
    connect_timeout = int(cfg.get("ssh_connect_timeout", 60))
    board_limit = connect_timeout + 15
    probe_rc, probe_out = board_run(
        term,
        f"timeout {board_limit} ssh $CI_SSH_OPTS $CI_SSH_HOST true </dev/null",
        timeout=float(board_limit + 30),
    )
    if probe_rc == 0:
        print("[network] ssh to CI host ok", flush=True)
        return
    _, pub_out = board_run(
        term,
        "cat ~/.ssh/id_ed25519.pub 2>/dev/null || true",
        timeout=15.0,
    )
    if not board_pubkey_from_output(pub_out):
        _, pub_out2 = board_run(term, "cat ~/.ssh/id_rsa.pub 2>/dev/null || true", timeout=15.0)
        pub_out = pub_out + pub_out2
    pubkey = board_pubkey_from_output(pub_out)
    if not pubkey:
        raise TerminalCommandError(
            f"ssh probe to {host_user}@{host_ip} failed and board public key not found\n"
            f"{probe_out.strip()}"
        )
    auth_keys = f"/home/{host_user}/.ssh/authorized_keys"
    raise TerminalCommandError(
        f"ssh probe to {host_user}@{host_ip} failed; add board public key to "
        f"{auth_keys} on the CI host:\n{pubkey}"
    )


def setup_board_network(cfg: dict[str, Any], term: Terminal) -> None:
    setup_board_interface(cfg, term)
    ping_host(cfg, term)
    probe_board_ssh(cfg, term)


def resolve_staging_dir(raw: str, arch: str, board: str) -> str:
    base = (raw or "/home/light/ci_deploy").rstrip("/")
    suffix = f"{arch}__{board.replace('/', '__')}"
    if base.endswith(suffix):
        return base
    return f"{base}/{suffix}"


def stage_board_files(cfg: dict[str, Any]) -> None:
    script = cfg["workspace"] / "jenkins" / "board_stage.sh"
    if not script.is_file():
        raise SystemExit(f"board stage script not found: {script}")
    env = os.environ.copy()
    env["ARCH"] = cfg["arch"]
    env["BOARD"] = cfg["board"]
    env["WORKSPACE_ROOT"] = str(cfg["workspace"])
    env["HVISOR_TOOL_PATH"] = cfg["hvisor_tool_path"]
    env["STAGING_DIR"] = cfg["staging_dir"]
    if cfg.get("zone1_dtb"):
        zone1_dtb = Path(cfg["zone1_dtb"])
        if not zone1_dtb.is_absolute():
            zone1_dtb = cfg["workspace"] / zone1_dtb
        env["ZONE1_DTB"] = str(zone1_dtb.resolve())
    subprocess.run(["bash", str(script)], check=True, cwd=cfg["workspace"], env=env)


def staging_alias(cfg: dict[str, Any]) -> str:
    alias = Path("/tmp/s") / cfg["board"].replace("/", "_")
    target = Path(cfg["staging_dir"])
    alias.parent.mkdir(parents=True, exist_ok=True)
    if alias.is_symlink() or alias.exists():
        alias.unlink()
    alias.symlink_to(target, target_is_directory=True)
    return str(alias)


def parse_staging(staging_path: Path) -> tuple[list[str], dict[str, list[str]], list[str]]:
    regular: list[str] = []
    split_groups: dict[str, list[str]] = {}
    for entry in staging_path.iterdir():
        if not entry.is_file():
            continue
        name = entry.name
        match = SPLIT_PART_RE.match(name)
        if match:
            split_groups.setdefault(match.group(1), []).append(name)
            continue
        regular.append(name)
    for base in split_groups:
        split_groups[base] = sorted(split_groups[base])
    parts = [part for names in split_groups.values() for part in names]
    pull_files = sorted(regular + parts, key=lambda n: staging_path.joinpath(n).stat().st_size)
    return regular, split_groups, pull_files


def file_pull_timeout(file_size: int, remaining: float) -> float:
    needed = max(90.0, file_size / 8192.0 + 45.0)
    return min(remaining, needed)


def ensure_scp_tmp(cfg: dict[str, Any], term: Terminal) -> None:
    board_run(term, f"mkdir -p {cfg['scp_tmp_dir']}", timeout=15.0)


def scp_one_to_tmp(
    cfg: dict[str, Any],
    term: Terminal,
    name: str,
    file_size: int,
    file_timeout: float,
    all_logs: list[str],
) -> None:
    tmp = cfg["scp_tmp_dir"]
    tmp_file = cfg["scp_tmp_file"]
    scp_limit = max(60, int(file_timeout) - 5)
    scp_cmd = (
        f"timeout {scp_limit} scp -q $CI_SSH_OPTS $CI_SSH_HOST:$CI_D/{name} "
        f"$CI_F </dev/null 2>/dev/null"
    )
    dest_in_tmp = f"{tmp}/{name}"
    try:
        rc, out = board_run_retry(term, scp_cmd, file_timeout, retries=0)
        if rc != 0:
            raise TerminalCommandError(f"board scp failed for {name!r}: {out.strip()}")
        rc, out = board_run_retry(term, f"cp {tmp_file} {dest_in_tmp}", 30.0, retries=0)
        if rc != 0:
            raise TerminalCommandError(f"board tmp install failed for {name!r}: {out.strip()}")
    except TerminalTimeoutError:
        check_tmp = f"test $(wc -c < {tmp_file}) -eq {file_size}"
        rc, out = board_run_retry(term, check_tmp, 30.0, retries=0)
        if rc != 0:
            all_logs.append(f"=== scp_{name} timeout ===\n{out}\n")
            raise
        rc, out = board_run_retry(term, f"cp {tmp_file} {dest_in_tmp}", 30.0, retries=0)
        if rc != 0:
            raise TerminalCommandError(f"board tmp install failed for {name!r}: {out.strip()}")
        print(f"[trans] scp recovered after timeout: {name}", flush=True)
    all_logs.append(f"=== scp_{name} ok ===\n")


def assemble_split_in_tmp(
    cfg: dict[str, Any],
    term: Terminal,
    staging_path: Path,
    base: str,
    parts: list[str],
    all_logs: list[str],
) -> None:
    tmp = cfg["scp_tmp_dir"]
    expected_size = sum(staging_path.joinpath(part).stat().st_size for part in parts)
    out_path = f"{tmp}/{base}"
    print(f"[trans] assemble {base} from {len(parts)} part(s) in tmp", flush=True)
    board_run_check(term, f"cp {tmp}/{parts[0]} {out_path}", timeout=60.0)
    for part in parts[1:]:
        board_run_check(term, f"cat {tmp}/{part} >> {out_path}", timeout=60.0)
    board_run_check(term, f"test $(wc -c < {out_path}) -eq {expected_size}", timeout=30.0)
    for part in parts:
        board_run(term, f"rm -f {tmp}/{part}", timeout=15.0)
    all_logs.append(f"=== assemble {base} ===\n")


def gunzip_in_tmp(cfg: dict[str, Any], term: Terminal, gz_name: str, all_logs: list[str]) -> None:
    raw_name = GUNZIP_ARTIFACTS[gz_name]
    raw_path = Path(cfg["hvisor_tool_path"]) / "output" / raw_name
    raw_size = raw_path.stat().st_size
    tmp = cfg["scp_tmp_dir"]
    print(f"[trans] gunzip in tmp: {gz_name} -> {raw_name}", flush=True)
    board_run_check(term, f"gunzip -f {tmp}/{gz_name}", timeout=120.0)
    board_run_check(term, f"test $(wc -c < {tmp}/{raw_name}) -eq {raw_size}", timeout=30.0)
    all_logs.append(f"=== gunzip {raw_name} ===\n")


def install_tmp_to_workdir(cfg: dict[str, Any], term: Terminal, names: list[str]) -> None:
    work_dir = cfg["zone1_work_dir"]
    tmp = cfg["scp_tmp_dir"]
    board_run(term, f"mkdir -p {work_dir}", timeout=15.0)
    for name in names:
        board_run_check(term, f"cp {tmp}/{name} {board_path(work_dir, name)}", timeout=60.0)
    for script in ("boot_zone1.sh", "hvisor", "check_serial.sh"):
        board_run(term, f"chmod +x {board_path(work_dir, script)} 2>/dev/null || true", timeout=15.0)
    print(f"[trans] installed {len(names)} file(s) to {work_dir}", flush=True)


def pull_to_tmp_and_install(cfg: dict[str, Any], term: Terminal) -> None:
    stage_board_files(cfg)
    staging_path = Path(cfg["staging_dir"])
    if not staging_path.is_dir():
        raise TerminalCommandError(f"staging dir not found: {staging_path}")
    staging_dir = staging_alias(cfg)
    ensure_scp_tmp(cfg, term)
    export_board_ssh_env(cfg, term)
    board_run(term, f"export CI_D={shlex.quote(staging_dir)}", timeout=15.0)
    board_run(term, f"export CI_T={shlex.quote(cfg['scp_tmp_dir'])}", timeout=15.0)
    board_run(term, f"export CI_F={shlex.quote(cfg['scp_tmp_file'])}", timeout=15.0)
    regular_files, split_groups, pull_files = parse_staging(staging_path)
    if not pull_files:
        raise TerminalCommandError(f"no staged files in {staging_path}")
    pull_timeout = float(cfg.get("pull_timeout", 600.0))
    deadline = time.monotonic() + pull_timeout
    all_logs: list[str] = []
    tmp = cfg["scp_tmp_dir"]
    stale = sorted(set(pull_files) | set(split_groups) | set(GUNZIP_ARTIFACTS.values()))
    for name in stale:
        board_run(term, f"rm -f {tmp}/{name}", timeout=15.0)
    print(f"[trans] pulling {len(pull_files)} file(s) to {tmp}", flush=True)
    for index, name in enumerate(pull_files, start=1):
        if index > 1:
            time.sleep(3.0)
        remaining = deadline - time.monotonic()
        if remaining <= 0:
            raise TerminalTimeoutError("pull timed out before completing scp")
        file_size = staging_path.joinpath(name).stat().st_size
        file_timeout = file_pull_timeout(file_size, remaining)
        print(f"[trans] scp ({index}/{len(pull_files)}): {name}", flush=True)
        scp_one_to_tmp(cfg, term, name, file_size, file_timeout, all_logs)
    for base, parts in sorted(split_groups.items()):
        assemble_split_in_tmp(cfg, term, staging_path, base, parts, all_logs)
    for gz_name in GUNZIP_ARTIFACTS:
        if gz_name in regular_files or gz_name in split_groups:
            gunzip_in_tmp(cfg, term, gz_name, all_logs)
    install_names = sorted(set(regular_files) | set(split_groups))
    for gz, raw in GUNZIP_ARTIFACTS.items():
        if gz in install_names:
            install_names.remove(gz)
            install_names.append(raw)
    install_tmp_to_workdir(cfg, term, install_names)
    save_case_log(cfg, "deploy_scp.log", "".join(all_logs))


def strip_board_markers(text: str) -> str:
    return BOARD_CMD_MARKER_RE.sub("", text)


def parse_boot_script_lines(content: str) -> list[str]:
    content = strip_board_markers(content)
    lines: list[str] = []
    for raw in content.splitlines():
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        if line.startswith("#!/"):
            continue
        if "__HV_M_" in line or "; echo " in line:
            continue
        lines.append(line)
    return lines


def board_zone_list_shows_running(
    term: Terminal,
    zone_name: str = "linux2",
) -> None:
    # hvisor-tool returns zone count (non-zero) from zone list; validate output instead.
    _, out = board_run(term, "./hvisor zone list", timeout=15.0)
    if zone_name not in out or "running" not in out:
        raise TerminalCommandError(
            f"zone list missing running {zone_name!r}:\n{out.strip()}"
        )


def boot_line_command(work_dir: str, line: str) -> str:
    stripped = line.rstrip()
    if stripped.startswith("cd "):
        inner = stripped
    else:
        inner = f"cd {work_dir} && {stripped}"
    if stripped.endswith("&"):
        return f"( {inner} )"
    return inner


def line_timeout(line: str) -> float:
    if "&" in line or "nohup" in line:
        return 30.0
    return 120.0


def find_zone1_pts(term: Terminal, timeout: float = 60.0) -> int:
    """Return the newest virtio-console pts number (poll ls until it appears)."""
    deadline = time.monotonic() + timeout
    last_output = ""
    while time.monotonic() < deadline:
        _, pts_output = board_run(term, "ls -1 /dev/pts/[0-9]*", timeout=15.0)
        last_output = pts_output
        pts_numbers = sorted(int(m) for m in re.findall(r"/dev/pts/(\d+)", pts_output))
        if pts_numbers:
            return pts_numbers[-1]
        time.sleep(1.0)
    raise TerminalCommandError(
        f"timed out waiting for zone1 pts device (last ls output: {last_output.strip()!r})"
    )


def run_boot_script_lines(cfg: dict[str, Any], term: Terminal) -> None:
    work_dir = cfg["zone1_work_dir"]
    boot_script = "boot_zone1.sh"
    _, content = board_run(term, f"cat {board_path(work_dir, boot_script)}", timeout=30.0)
    lines = parse_boot_script_lines(content)
    if not lines:
        raise TerminalCommandError(f"no executable lines in {boot_script}")
    print(f"[zone1] running {len(lines)} line(s) from {boot_script}", flush=True)
    for index, line in enumerate(lines, start=1):
        cmd = boot_line_command(work_dir, line)
        if len(cmd) > MAX_BOARD_CMD_LEN:
            raise TerminalCommandError(
                f"boot line {index} too long ({len(cmd)} > {MAX_BOARD_CMD_LEN}): {line!r}"
            )
        try:
            board_run_check(term, cmd, timeout=line_timeout(line))
        except TerminalCommandError as exc:
            raise TerminalCommandError(f"boot line {index} failed: {line}\n{exc}") from exc


def boot_board_zone0_with_retry(cfg: dict[str, Any], term: Terminal) -> None:
    retry_step(
        "zone0_boot",
        lambda: boot_board_zone0(cfg, term),
        retries=int(cfg.get("retry_zone0", 2)),
    )


def board_login(cfg: dict[str, Any], term: Terminal) -> int:
    prepare_board_shell(cfg, term)
    check_login_tools(cfg, term)
    return 0


def board_network_and_trans(cfg: dict[str, Any], term: Terminal) -> int:
    def run_trans() -> None:
        setup_board_network(cfg, term)
        pull_to_tmp_and_install(cfg, term)

    def on_network_retry() -> None:
        term.send("\x03")
        time.sleep(1.0)
        board_wait_shell(term, cfg, timeout=30.0)

    retry_step(
        "network_and_trans",
        run_trans,
        retries=int(cfg.get("retry_network", 2)),
        on_retry=on_network_retry,
    )
    return 0


def board_zone1_stop(cfg: dict[str, Any], term: Terminal) -> None:
    """Stop zone1 before a retry (hvisor-tool: zone shutdown -id <id>)."""
    work_dir = cfg["zone1_work_dir"]
    zone_id = int(cfg.get("zone1_id", 1))
    board_run(term, f"cd {work_dir}", timeout=15.0)
    rc, out = board_run(term, f"./hvisor zone shutdown -id {zone_id}", timeout=60.0)
    if rc != 0:
        print(
            f"[zone1] shutdown rc={rc} (zone may not be running): {out.strip()}",
            flush=True,
        )
    board_wait_shell(term, cfg, timeout=30.0)


def board_zone1_start(cfg: dict[str, Any], term: Terminal) -> int:
    work_dir = cfg["zone1_work_dir"]
    inner_log = "/tmp/zone1_inner.log"

    def run_zone1() -> None:
        board_run(term, f"cd {work_dir}", timeout=15.0)
        board_run(term, "ls", timeout=15.0)
        run_boot_script_lines(cfg, term)
        board_zone_list_shows_running(
            term, str(cfg.get("zone1_name", "linux2"))
        )
        max_pts = find_zone1_pts(term)
        board_run(term, f"cd {work_dir}", timeout=15.0)
        check_rc, _ = board_run(
            term,
            f"./check_serial.sh /dev/pts/{max_pts} {inner_log} {int(ZONE1_INNER_PROMPT_TIMEOUT)}",
            timeout=ZONE1_INNER_PROMPT_TIMEOUT + 30.0,
        )
        if check_rc != 0:
            raise TerminalCommandError("check_serial.sh failed (no console prompt)")
        _, inner_output = board_run(term, f"tail -c 131072 {inner_log}", timeout=30.0)
        if inner_output:
            save_case_log(cfg, "zone1_inner_serial.log", inner_output)

    retry_step(
        "zone1_start",
        run_zone1,
        retries=int(cfg.get("retry_zone1", 1)),
        on_retry=lambda: board_zone1_stop(cfg, term),
    )
    print("zone1_started successfully", flush=True)
    return 0
