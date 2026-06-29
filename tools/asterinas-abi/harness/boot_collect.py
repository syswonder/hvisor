#!/usr/bin/env python3
# SPDX-License-Identifier: MulanPSL-2.0
"""Boot an Asterinas autorun ISO (or a Linux bzImage) under QEMU, capture the
serial console, decode the hex-framed ABI result JSON, and write it out.

This is the unattended Env A / reference-Linux collector. It streams the console
to a log, watches for the ABI-DONE marker, and enforces a hard wall-clock cap.
QEMU, accelerator and OVMF paths are configurable (see --help).
"""

from __future__ import annotations

import argparse
import binascii
import json
import os
import re
import select
import signal
import shutil
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

# QEMU and OVMF default to the system install; override via flags or the
# ABI_QEMU / ABI_OVMF_CODE / ABI_OVMF_VARS environment variables. Booting an
# Asterinas image directly needs QEMU >= 9.x; QEMU 6.2 only runs hvisor.
QEMU_DEFAULT = os.environ.get("ABI_QEMU") or shutil.which("qemu-system-x86_64") \
    or "qemu-system-x86_64"
OVMF_CODE_DEFAULT = os.environ.get("ABI_OVMF_CODE", "/usr/share/OVMF/OVMF_CODE.fd")
OVMF_VARS_DEFAULT = os.environ.get("ABI_OVMF_VARS", "/usr/share/OVMF/OVMF_VARS.fd")

BEGIN = "===ABI-RESULTS-BEGIN==="
END = "===ABI-RESULTS-END==="
DONE = "===ABI-DONE==="
HEX_RE = re.compile(r"ABIHEX:([0-9a-fA-F ]+)")


def strip_ansi(text: str) -> str:
    return re.sub(r"\x1b\[[0-9;?]*[a-zA-Z]", "", text)


def decode_hex_frame(console_text: str) -> str | None:
    """Pull the hex-framed JSON out of the console capture and decode it."""
    if BEGIN not in console_text:
        return None
    frame = console_text.split(BEGIN, 1)[1]
    if END in frame:
        frame = frame.split(END, 1)[0]
    hex_bytes = bytearray()
    for line in frame.splitlines():
        m = HEX_RE.search(line)
        if not m:
            continue
        hexchars = m.group(1).replace(" ", "").strip()
        if not hexchars:
            continue
        try:
            hex_bytes.extend(binascii.unhexlify(hexchars))
        except binascii.Error:
            # Tolerate a truncated final line.
            trimmed = hexchars[: len(hexchars) - (len(hexchars) % 2)]
            try:
                hex_bytes.extend(binascii.unhexlify(trimmed))
            except binascii.Error:
                continue
    if not hex_bytes:
        return None
    try:
        return hex_bytes.decode("utf-8", errors="replace")
    except Exception:
        return None


def build_qemu_cmd(args, vars_path: Path) -> list[str]:
    if args.mode == "linux":
        # Direct Linux bzImage boot: QEMU's -kernel supports the Linux boot
        # protocol natively, no firmware needed. Same initramfs + same autorun.
        # The `pc` machine wires an ISA COM1 the stock kernel drives reliably;
        # the explicit baud is required for output on ttyS0.
        cmd = [
            args.qemu,
            "-machine", f"pc,accel={args.accel}",
            "-cpu", args.cpu,
            "-smp", str(args.smp),
            "-m", args.mem,
            "-kernel", str(args.kernel),
            "-initrd", str(args.initrd),
            "-append", args.append,
            "-nographic", "-display", "none",
            "-serial", "mon:stdio",
            "-no-reboot",
        ]
        return cmd
    # iso mode (Asterinas grub-rescue + OVMF, multiboot2)
    cmd = [
        args.qemu,
        "-machine", f"q35,accel={args.accel},kernel-irqchip=split",
        "-cpu", args.cpu,
        "-smp", str(args.smp),
        "-m", args.mem,
        "-drive", f"if=pflash,format=raw,readonly=on,file={args.ovmf_code}",
        "-drive", f"if=pflash,format=raw,file={vars_path}",
        "-cdrom", str(args.iso),
        "-nographic", "-display", "none",
        "-serial", "mon:stdio",
        "-no-reboot",
    ]
    if args.netdev:
        cmd += ["-netdev", "user,id=net0", "-device", "virtio-net-pci,netdev=net0"]
    return cmd


def qemu_version(qemu: str) -> str:
    try:
        proc = subprocess.run(
            [qemu, "--version"],
            check=False,
            capture_output=True,
            text=True,
            timeout=5,
        )
    except (OSError, subprocess.TimeoutExpired):
        return Path(qemu).name
    first = (proc.stdout or proc.stderr).splitlines()
    return first[0] if first else Path(qemu).name


def should_echo_console_line(line: str) -> bool:
    clean = strip_ansi(line).rstrip()
    if not clean or "ABIHEX:" in clean:
        return False
    return (
        "ABI-" in clean
        or clean.startswith("[")
        or "OSTD" in clean
        or "panic" in clean.lower()
        or "unpacking" in clean
    )


def main(argv: list[str]) -> int:
    p = argparse.ArgumentParser()
    p.add_argument("--mode", choices=["iso", "linux"], default="iso",
                   help="iso = Asterinas grub-rescue+OVMF; linux = direct bzImage -kernel boot")
    p.add_argument("--iso", type=Path, help="ISO for iso mode")
    p.add_argument("--kernel", type=Path, help="bzImage for linux mode")
    p.add_argument("--initrd", type=Path, help="initramfs for linux mode")
    p.add_argument("--append", default="console=ttyS0 init=/init",
                   help="kernel cmdline for linux mode")
    p.add_argument("--env-id", default="A", help="environment id label (A/B/C/ref_linux/...)")
    p.add_argument("--out", type=Path, required=True, help="results.json destination")
    p.add_argument("--console-log", type=Path, default=None)
    p.add_argument("--qemu", default=QEMU_DEFAULT, help="qemu-system-x86_64 to use")
    p.add_argument("--accel", default="kvm", help="QEMU accelerator (kvm or tcg)")
    p.add_argument("--ovmf-code", default=OVMF_CODE_DEFAULT, help="OVMF_CODE.fd (iso mode)")
    p.add_argument("--ovmf-vars", default=OVMF_VARS_DEFAULT, help="OVMF_VARS.fd (iso mode)")
    p.add_argument("--cpu", default="host")
    p.add_argument("--smp", type=int, default=4)
    p.add_argument("--mem", default="4G")
    p.add_argument("--netdev", action="store_true")
    p.add_argument("--max-seconds", type=int, default=1800)
    p.add_argument("--label", default="Asterinas on QEMU")
    args = p.parse_args(argv)

    qemu_path = shutil.which(args.qemu) or (args.qemu if Path(args.qemu).exists() else None)
    if not qemu_path:
        print(f"ERROR: qemu-system-x86_64 not found: {args.qemu}", file=sys.stderr)
        return 2
    args.qemu = qemu_path
    if args.mode == "iso" and (not args.iso or not args.iso.exists()):
        print(f"ERROR: ISO not found: {args.iso}", file=sys.stderr)
        return 2
    if args.mode == "linux" and (not args.kernel or not args.kernel.exists()
                                 or not args.initrd or not args.initrd.exists()):
        print(f"ERROR: linux mode needs --kernel and --initrd", file=sys.stderr)
        return 2

    vars_path = ROOT / "_build" / f"ovmf_vars_run_{args.env_id}.fd"
    if args.mode == "iso":
        vars_path.parent.mkdir(parents=True, exist_ok=True)
        vars_path.write_bytes(Path(args.ovmf_vars).read_bytes())

    console_log = args.console_log or (ROOT / "results" / f"env_{args.env_id.lower()}" / "console.log")
    console_log.parent.mkdir(parents=True, exist_ok=True)

    cmd = build_qemu_cmd(args, vars_path)
    env = os.environ.copy()
    # A QEMU built into a private sysroot needs its shared libraries on the
    # path; a system QEMU does not. Only prepend when such a sysroot exists.
    sysroot = Path(os.environ.get("ABI_QEMU_SYSROOT", "")) if os.environ.get("ABI_QEMU_SYSROOT") else None
    if sysroot and sysroot.exists():
        env["LD_LIBRARY_PATH"] = (
            f"{sysroot}/usr/lib/x86_64-linux-gnu:{sysroot}/lib/x86_64-linux-gnu:"
            + env.get("LD_LIBRARY_PATH", "")
        )

    print(f"[collect] env={args.env_id} cpu={args.cpu} smp={args.smp} mem={args.mem}")
    print(f"[collect] iso={args.iso}")
    print(f"[collect] console log -> {console_log}")
    print(f"[collect] launching QEMU (cap {args.max_seconds}s)...")

    start = time.monotonic()
    captured = []
    done_seen = False
    with console_log.open("w", encoding="utf-8", errors="replace") as logf:
        proc = subprocess.Popen(
            cmd, stdin=subprocess.DEVNULL, stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT, bufsize=0, env=env,
        )
        try:
            assert proc.stdout is not None
            os.set_blocking(proc.stdout.fileno(), False)
            pending = ""
            while proc.poll() is None:
                now = time.monotonic()
                if now - start > args.max_seconds:
                    print("[collect] hard time cap reached; stopping guest.")
                    break
                timeout = min(0.5, max(0.0, args.max_seconds - (now - start)))
                ready, _, _ = select.select([proc.stdout], [], [], timeout)
                if not ready:
                    continue
                chunk = os.read(proc.stdout.fileno(), 4096)
                if not chunk:
                    break
                text = chunk.decode("utf-8", errors="replace")
                logf.write(text)
                logf.flush()
                captured.append(text)
                pending += text
                done_in_chunk = DONE in pending
                while "\n" in pending:
                    line, pending = pending.split("\n", 1)
                    if should_echo_console_line(line):
                        print(f"  | {strip_ansi(line).rstrip()[:120]}")
                if done_in_chunk:
                    done_seen = True
                    print("[collect] ABI-DONE marker seen; stopping guest.")
                    break
            if pending and should_echo_console_line(pending):
                print(f"  | {strip_ansi(pending).rstrip()[:120]}")
        finally:
            if proc.poll() is None:
                proc.send_signal(signal.SIGTERM)
                try:
                    proc.wait(timeout=8)
                except subprocess.TimeoutExpired:
                    proc.kill()
                    proc.wait()
            # Drain any remaining buffered output.
            if proc.stdout is not None:
                try:
                    rest = proc.stdout.read()
                    if rest:
                        text = rest.decode("utf-8", errors="replace")
                        logf.write(text)
                        captured.append(text)
                except Exception:
                    pass

    elapsed = time.monotonic() - start
    console_text = strip_ansi("".join(captured))
    print(f"[collect] guest ran {elapsed:.0f}s; done_marker={done_seen}")

    decoded = decode_hex_frame(console_text)
    if decoded is None:
        print("[collect] ERROR: no ABI result frame decoded from console.", file=sys.stderr)
        print(f"[collect] inspect console log: {console_log}", file=sys.stderr)
        return 1

    try:
        data = json.loads(decoded)
    except json.JSONDecodeError as exc:
        raw = console_log.parent / f"results_raw_{args.env_id.lower()}.json"
        raw.write_text(decoded, encoding="utf-8")
        print(f"[collect] ERROR: decoded frame is not valid JSON ({exc}).", file=sys.stderr)
        print(f"[collect] raw decode saved to {raw}", file=sys.stderr)
        return 1

    meta = data.setdefault("metadata", {})
    meta["env_id"] = args.env_id
    meta["label"] = args.label
    meta["measurement_status"] = "measured"
    meta["wall_clock_seconds"] = round(elapsed, 1)
    meta["accel"] = args.accel
    meta["qemu"] = qemu_version(args.qemu)

    args.out.parent.mkdir(parents=True, exist_ok=True)
    with args.out.open("w", encoding="utf-8") as f:
        json.dump(data, f, indent=2, sort_keys=True)
        f.write("\n")

    summary = data.get("summary", {})
    print(f"[collect] OK -> {args.out}")
    print(f"[collect] summary: {summary}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
