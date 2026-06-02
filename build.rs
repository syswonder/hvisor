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
