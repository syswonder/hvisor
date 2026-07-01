// Copyright (c) 2025 Syswonder
// hvisor is licensed under Mulan PSL v2.
// You can use this software according to the terms and conditions of the Mulan PSL v2.
// You may obtain a copy of Mulan PSL v2 at:
//     http://license.coscl.org.cn/MulanPSL2
// THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER
// EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR
// FIT FOR A PARTICULAR PURPOSE.
// See the Mulan PSL v2 for more details.
//
// Syswonder Website:
//      https://www.syswonder.org
//
// Authors:
//      Yulong Han <wheatfox17@icloud.com>
//
use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::{env, fs, path::Path};

const BUILD_LOG_FILE: &str = "target/build_rs.log";

fn log(output: &str) {
    let log_path = Path::new(BUILD_LOG_FILE);
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)
        .expect("Failed to open log file");
    writeln!(file, "{}", output).expect("Failed to write to log file");
}

struct BuildEnv {
    arch: String,
    board: String,
    bid: String,
}

// parse ARCH, BOARD and BID from .config
fn parse_build_env(file_path: &str) -> BuildEnv {
    let file = fs::read_to_string(file_path).expect("Failed to read .config file");
    let mut arch = String::new();
    let mut board = String::new();
    let mut bid = String::new();
    for line in file.lines() {
        let t = line.trim();
        if let Some(v) = t.strip_prefix("# ARCH=") {
            arch = v.to_string();
            continue;
        }
        if let Some(v) = t.strip_prefix("# BOARD=") {
            board = v.to_string();
            continue;
        }
        if let Some(v) = t.strip_prefix("# BID=") {
            bid = v.to_string();
            continue;
        }
        if t.starts_with('#') {
            continue;
        }
        let Some((k, v)) = t.split_once('=') else {
            continue;
        };
        match k {
            "ARCH" if arch.is_empty() => arch = v.to_string(),
            "BOARD" if board.is_empty() => board = v.to_string(),
            "BID" if bid.is_empty() => bid = v.to_string(),
            _ => {}
        }
    }
    BuildEnv { arch, board, bid }
}

fn load_cfg_map(root: &Path) -> HashMap<String, String> {
    let path = root.join("kconfig/cfg_map.toml");
    let raw = fs::read_to_string(&path).unwrap_or_else(|_| panic!("missing {}", path.display()));
    let mut m = HashMap::new();
    let mut in_symbols = false;
    for line in raw.lines() {
        let t = line.trim();
        if t.is_empty() || t.starts_with('#') {
            continue;
        }
        if t == "[symbols]" {
            in_symbols = true;
            continue;
        }
        if t.starts_with('[') {
            in_symbols = false;
            continue;
        }
        if !in_symbols {
            continue;
        }
        let Some((k, v)) = t.split_once('=') else {
            continue;
        };
        let k = k.trim().trim_matches('"');
        let v = v.trim().trim_matches('"');
        m.insert(k.to_string(), v.to_string());
    }
    if m.is_empty() {
        panic!("cfg_map.toml: no entries under [symbols]");
    }
    m
}

// parse enabled config keys from .config
fn parse_enabled_config_keys(config_text: &str) -> HashSet<String> {
    let mut out = HashSet::new();
    for line in config_text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, val)) = line.split_once('=') else {
            continue;
        };
        let val = val.trim();
        if val != "y" && val != "m" {
            continue;
        }
        if key.starts_with("CONFIG_") {
            out.insert(key.to_string());
        }
    }
    out
}

fn main() {
    let log_path = Path::new(BUILD_LOG_FILE);
    if log_path.exists() {
        fs::remove_file(log_path).expect("Failed to remove log file");
    }

    let project_toml_root = env::var("CARGO_MANIFEST_DIR").unwrap();
    let root = Path::new(&project_toml_root);
    let config_path = root.join(".config");
    let mut build_env = parse_build_env(config_path.to_str().unwrap());

    if !build_env.bid.is_empty() {
        let parts: Vec<&str> = build_env.bid.split('/').collect();
        if parts.len() != 2 {
            log(&format!("Invalid BID format: {}", build_env.bid));
            panic!(
                "Invalid BID format, please check the log file({}) for more details",
                BUILD_LOG_FILE
            );
        }
        build_env.arch = parts[0].to_string();
        build_env.board = parts[1].to_string();
    } else if build_env.arch.is_empty() || build_env.board.is_empty() {
        log("ARCH or BOARD missing in .config");
        panic!(
            "ARCH or BOARD missing in .config (run: make defconfig), see {}",
            BUILD_LOG_FILE
        );
    }

    let arch = build_env.arch;
    let board = build_env.board;
    let bid = build_env.bid;

    let pwd = env::current_dir().unwrap();
    log(&format!("Current directory: {}", pwd.display()));

    let cfg_map = load_cfg_map(root);
    let config_text = fs::read_to_string(&config_path).expect("read .config");
    let enabled = parse_enabled_config_keys(&config_text);

    for rust_cfg in cfg_map.values().collect::<HashSet<_>>() {
        println!("cargo:rustc-check-cfg=cfg({rust_cfg})");
    }

    for k in &enabled {
        if let Some(rust_cfg) = cfg_map.get(k) {
            println!("cargo:rustc-cfg={rust_cfg}");
        } else if k.starts_with("CONFIG_") {
            log(&format!("warning: unknown config key in .config: {k}"));
        }
    }

    let target_path_str = format!("{}/src/platform/__board.rs", pwd.display());
    let target_path = Path::new(&target_path_str);
    let source_path_str = format!("{}/platform/{}/{}/board.rs", pwd.display(), arch, board);
    let source_path = Path::new(&source_path_str);

    log(&format!(
        "Building for ARCH={arch} BOARD={board}, BID={bid}, enabled_cfgs={}",
        enabled.len()
    ));

    log(&format!(
        "Linking board.rs from {} to {}",
        source_path_str,
        target_path.display()
    ));

    if !source_path.exists() {
        log(&format!("Invalid board.rs path: {source_path_str}"));
        panic!("Invalid board.rs, please check the log file({BUILD_LOG_FILE}) for more details");
    }

    if target_path.exists() {
        fs::remove_file(target_path).expect("Failed to remove existing __board.rs");
    }
    std::os::unix::fs::symlink(source_path, target_path).expect("Failed to create symlink");
    log("Linking successful");

    // When CONFIG_ASTERINAS_ROOT is enabled, capture the byte length of
    // the staged *uncompressed* initramfs so the root-zone boot params can
    // advertise the true ramdisk size without a hand-edited constant. The value
    // is written to OUT_DIR and `include!`d by board.rs. It is 0 (and the boot
    // params carry no ramdisk) when the option is off or the file is not staged
    // yet; staging the initramfs then triggers a rebuild via rerun-if-changed.
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");
    let initrd_rs_path = format!("{}/asterinas_initrd.rs", out_dir);
    let initrd_size: u64 = if enabled.contains("CONFIG_ASTERINAS_ROOT") {
        let img = format!("{}/platform/x86_64/qemu/image", pwd.display());
        let setup_path = format!("{}/kernel/asterinas-setup.bin", img);
        let vmlinux_path = format!("{}/kernel/asterinas-vmlinux.bin", img);
        let initrd_path = format!("{}/virtdisk/initramfs.cpio", img);
        for p in [&setup_path, &vmlinux_path, &initrd_path] {
            println!("cargo:rerun-if-changed={}", p);
        }
        // The asterinas_root boot requires all payload files to be staged.
        // Fail the build rather than emit a non-bootable ISO with a zero
        // ramdisk size: the Linux/x86 boot protocol needs `ramdisk_size` to be
        // the true initrd byte length, and the GRUB entry needs the split kernel.
        let require = |p: &str| -> u64 {
            match fs::metadata(p) {
                Ok(m) if m.len() > 0 => m.len(),
                _ => panic!(
                    "asterinas_root feature is enabled but the boot artifact `{}` is missing or \
                     empty. Stage asterinas-setup.bin, asterinas-vmlinux.bin, and an uncompressed \
                     initramfs.cpio before building.",
                    p
                ),
            }
        };
        require(&setup_path);
        require(&vmlinux_path);
        let initrd_len = require(&initrd_path);
        // Asterinas consumes the ramdisk as a raw newc cpio. A gzip-compressed
        // or otherwise non-cpio file at this path would pass the existence check
        // but fail during guest unpack, so verify the newc magic here.
        let head = fs::read(&initrd_path).unwrap_or_default();
        let magic = &head[..head.len().min(6)];
        if magic != b"070701" && magic != b"070702" {
            panic!(
                "asterinas_root: `{}` is not an uncompressed newc cpio (leading bytes {:?}). \
                 Asterinas consumes a raw cpio; use `gzip -dc initramfs.cpio.gz > initramfs.cpio`.",
                initrd_path, magic
            );
        }
        // The Linux/x86 boot protocol conveys the ramdisk via the 32-bit
        // `ramdisk_size` field, so an initramfs above u32::MAX would silently
        // truncate the boot contract; reject it at build time.
        if initrd_len > u32::MAX as u64 {
            panic!(
                "asterinas_root: initramfs is {} bytes, larger than the 32-bit Linux/x86 \
                 boot-protocol ramdisk_size field can represent.",
                initrd_len
            );
        }
        initrd_len
    } else {
        0
    };
    fs::write(
        initrd_rs_path,
        format!(
            "// Auto-generated by build.rs; do not edit.\n\
             #[allow(dead_code)]\n\
             pub const ASTERINAS_ROOT_INITRD_SIZE: usize = {};\n",
            initrd_size
        ),
    )
    .expect("Failed to write generated asterinas_initrd.rs");

    println!("cargo:rerun-if-env-changed=ARCH");
    println!("cargo:rerun-if-env-changed=BOARD");
    println!("cargo:rerun-if-env-changed=BID");
    println!("cargo:rerun-if-changed={}", config_path.display());
    println!(
        "cargo:rerun-if-changed={}",
        root.join("kconfig/cfg_map.toml").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        root.join("kconfig/Kconfig").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        root.join("tools/kconfig/kconfig_cli.py").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        root.join("platform")
            .join(&arch)
            .join(&board)
            .join("kconfig/defconfig")
            .display()
    );
    println!("cargo:rerun-if-changed={source_path_str}");
    println!("cargo:rerun-if-changed={target_path_str}");
}
