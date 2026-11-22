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
use std::{
    env, fs,
    path::{Path, PathBuf},
};

const BUILD_LOG_FILE: &str = "build_rs.log";

fn build_log_path() -> PathBuf {
    env::var_os("OUT_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("target"))
        .join(BUILD_LOG_FILE)
}

fn log(output: &str) {
    let log_path = build_log_path();
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .unwrap_or_else(|e| panic!("Failed to open {}: {e}", log_path.display()));
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
    let log_path = build_log_path();
    if log_path.exists() {
        fs::remove_file(&log_path).expect("Failed to remove log file");
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
                log_path.display()
            );
        }
        if (!build_env.arch.is_empty() && build_env.arch != parts[0])
            || (!build_env.board.is_empty() && build_env.board != parts[1])
        {
            panic!(
                ".config BID {} disagrees with ARCH={} BOARD={}; run make defconfig",
                build_env.bid, build_env.arch, build_env.board
            );
        }
        build_env.arch = parts[0].to_string();
        build_env.board = parts[1].to_string();
    } else if build_env.arch.is_empty() || build_env.board.is_empty() {
        log("ARCH or BOARD missing in .config");
        panic!(
            "ARCH or BOARD missing in .config (run: make defconfig), see {}",
            log_path.display()
        );
    }

    let arch = build_env.arch;
    let board = build_env.board;
    let bid = build_env.bid;

    let encoded_rustflags = env::var("CARGO_ENCODED_RUSTFLAGS").unwrap_or_default();
    if encoded_rustflags
        .split('\x1f')
        .any(|flag| flag.starts_with("-T") || flag.contains("link-arg=-T"))
    {
        panic!(
            "stale -T in rustflags; remove -Clink-arg=-T from platform/*/*/cargo/config.template.toml"
        );
    }

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

    let source_path = root
        .join("platform")
        .join(&arch)
        .join(&board)
        .join("board.rs");
    let source_path_str = source_path.display().to_string();

    log(&format!(
        "Building for ARCH={arch} BOARD={board}, BID={bid}, enabled_cfgs={}",
        enabled.len()
    ));

    // Board-specific linker arguments must not live in RUSTFLAGS. Cargo hashes
    // RUSTFLAGS into every dependency unit, so a board switch would invalidate
    // all dependency artifacts. rustc-link-arg only affects this package's
    // final link and is therefore safe to vary per board.
    let ld_script = root
        .join("platform")
        .join(&arch)
        .join(&board)
        .join("linker.ld");
    if !ld_script.is_file() {
        panic!("missing linker script: {}", ld_script.display());
    }
    println!("cargo:rustc-link-arg=-T{}", ld_script.display());
    if matches!(
        (arch.as_str(), board.as_str()),
        ("aarch64", "zcu102") | ("aarch64", "imx8mp") | ("aarch64", "ok6254-c")
    ) {
        println!("cargo:rustc-link-arg=--fix-cortex-a53-843419");
    }
    println!("cargo:rerun-if-changed={}", ld_script.display());

    if !source_path.is_file() {
        log(&format!("Invalid board.rs path: {source_path_str}"));
        panic!(
            "Invalid board.rs, please check the log file({}) for more details",
            log_path.display()
        );
    }

    // Generate the board module in OUT_DIR for the make-driven build.
    // This keeps the source tree immutable.
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"));
    let generated = out_dir.join("board.rs");
    fs::copy(&source_path, &generated).unwrap_or_else(|e| {
        panic!(
            "failed to copy {} to {}: {e}",
            source_path.display(),
            generated.display()
        )
    });
    log(&format!(
        "Generated board module {} from {}",
        generated.display(),
        source_path_str
    ));

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
}
