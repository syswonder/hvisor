#!/usr/bin/env python3
"""Kconfig entrypoints: defconfig, menuconfig, vscode-cfgs."""
from __future__ import annotations

import argparse
import json
import os
import re
import sys
import tomllib
from pathlib import Path

_KCONF_ARCH_TO_DIR: dict[str, str] = {
    "ARCH_AARCH64": "aarch64",
    "ARCH_RISCV64": "riscv64",
    "ARCH_LOONGARCH64": "loongarch64",
    "ARCH_X86_64": "x86_64",
}


def arch_from_kconf(kconf) -> str | None:
    for sym_name, arch in _KCONF_ARCH_TO_DIR.items():
        sym = kconf.syms.get(sym_name)
        if sym is not None and sym.str_value == "y":
            return arch
    return None


def write_root_dot_config(root: Path, arch: str, board: str, kconf) -> None:
    """Write repo-root `.config`: comment metadata (Kconfig-safe) then kconfiglib body."""
    cfg = root / ".config"
    derived = arch_from_kconf(kconf)
    if derived is None:
        raise SystemExit(
            "Kconfig: no architecture selected; enable exactly one of "
            "ARCH_AARCH64 / ARCH_RISCV64 / ARCH_LOONGARCH64 / ARCH_X86_64 in defconfig."
        )
    if derived != arch:
        raise SystemExit(
            f"Kconfig architecture ({derived}) does not match "
            f"platform path arch ({arch}) for board {board!r}. "
            "Fix kconfig/defconfig or pick the correct ARCH/BOARD."
        )
    arch_out = derived
    ld = (root / "platform" / arch_out / board / "linker.ld").resolve()
    tmpl = (root / "platform" / arch_out / board / "cargo" / "config.template.toml").resolve()
    hvisor_src = str(root.resolve())
    bid = f"{arch_out}/{board}"
    kconf.write_config(str(cfg))
    body = cfg.read_text()
    meta = (
        f"# ARCH={arch_out}\n"
        f"# BOARD={board}\n"
        f"# BID={bid}\n"
        f"# HVISOR_SRC={hvisor_src}\n"
        f"# LD_SCRIPT={ld}\n"
        f"# TEMPLATE={tmpl}\n"
        "\n"
    )
    cfg.write_text(meta + body)


def _repo_root() -> Path:
    return Path(__file__).resolve().parents[2]


def cmd_defconfig() -> None:
    arch = os.environ.get("ARCH", "")
    board = os.environ.get("BOARD", "")
    if not arch or not board:
        print("error: set ARCH and BOARD", file=sys.stderr)
        sys.exit(2)
    root = _repo_root()
    os.chdir(root)
    try:
        from kconfiglib import Kconfig
    except ImportError:
        print("error: pip install kconfiglib (see tools/kconfig/requirements.txt)", file=sys.stderr)
        sys.exit(1)
    kconf = Kconfig(str(root / "kconfig" / "Kconfig"))
    defcfg = root / "platform" / arch / board / "kconfig" / "defconfig"
    if not defcfg.is_file():
        print(f"error: missing {defcfg}", file=sys.stderr)
        sys.exit(1)
    kconf.load_config(str(defcfg))
    write_root_dot_config(root, arch, board, kconf)
    print(f"wrote .config for {arch}/{board}")


def cmd_menuconfig() -> None:
    arch = os.environ.get("ARCH", "")
    board = os.environ.get("BOARD", "")
    if not arch or not board:
        print("error: set ARCH and BOARD", file=sys.stderr)
        sys.exit(2)
    root = _repo_root()
    os.chdir(root)
    try:
        from kconfiglib import Kconfig
    except ImportError as e:
        print(f"error: kconfiglib is not installed: {e}", file=sys.stderr)
        print("hint: tools/kconfig/.venv/bin/pip install -r tools/kconfig/requirements.txt", file=sys.stderr)
        sys.exit(1)
    try:
        import menuconfig as kc_menu
    except ImportError as e:
        print(f"error: kconfiglib TUI (menuconfig) could not load: {e}", file=sys.stderr)
        print(
            "hint: the UI needs Python stdlib curses. "
            "Debian/Ubuntu: apt install python3-curses; Fedora: dnf install python3-curses; "
            "Windows: pip install windows-curses",
            file=sys.stderr,
        )
        sys.exit(1)
    kconf = Kconfig(str(root / "kconfig" / "Kconfig"))
    cfg = root / ".config"
    defcfg = root / "platform" / arch / board / "kconfig" / "defconfig"
    if cfg.is_file():
        kconf.load_config(str(cfg))
    elif defcfg.is_file():
        kconf.load_config(str(defcfg))
    kc_menu.menuconfig(kconf)
    write_root_dot_config(root, arch, board, kconf)


def cmd_vscode_cfgs(cfg_path: Path, map_path: Path) -> None:
    sym = tomllib.loads(map_path.read_text(encoding="utf-8"))["symbols"]
    enabled: set[str] = set()
    for line in cfg_path.read_text().splitlines():
        line = line.strip()
        m = re.match(r"^(CONFIG_[A-Z0-9_]+)=(y|m)$", line)
        if not m:
            continue
        k = m.group(1)
        if k in sym:
            enabled.add(sym[k])
    print(json.dumps(sorted(enabled)))


def main() -> int:
    p = argparse.ArgumentParser(description="hvisor Kconfig helpers")
    sub = p.add_subparsers(dest="cmd", required=True)

    sub.add_parser("defconfig", help="merge board defconfig + Kconfig into repo-root .config (env ARCH, BOARD)")
    sub.add_parser(
        "menuconfig",
        help="interactive Kconfig UI (env ARCH, BOARD); do not name this file menuconfig.py",
    )
    p_vc = sub.add_parser("vscode-cfgs", help="emit JSON array of rustc cfg names for rust-analyzer")
    p_vc.add_argument("config", type=Path, help="path to repo-root .config")
    p_vc.add_argument("map", type=Path, help="path to kconfig/cfg_map.toml")

    args = p.parse_args()
    if args.cmd == "defconfig":
        cmd_defconfig()
    elif args.cmd == "menuconfig":
        cmd_menuconfig()
    elif args.cmd == "vscode-cfgs":
        cmd_vscode_cfgs(args.config, args.map)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
