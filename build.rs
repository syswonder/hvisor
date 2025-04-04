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

// .config
// ARCH=...
// BOARD=...
struct BuildEnv {
    arch: String,
    board: String,
    bid: String,
    features: String,
}

fn parse_build_env(file_path: &str) -> BuildEnv {
    let file = fs::read_to_string(file_path).expect("Failed to read .config file");
    let mut arch = String::new();
    let mut board = String::new();
    let mut bid = String::new();
    let mut features = String::new();
    for line in file.lines() {
        let parts: Vec<&str> = line.split('=').collect();
        if parts.len() != 2 {
            continue;
        }
        match parts[0] {
            "ARCH" => arch = parts[1].to_string(),
            "BOARD" => board = parts[1].to_string(),
            "BID" => bid = parts[1].to_string(),
            "FEATURES" => features = parts[1].to_string(),
            _ => {}
        }
    }
    BuildEnv {
        arch,
        board,
        bid,
        features,
    }
}

fn main() {
    // cleanup the log file
    let log_path = Path::new(BUILD_LOG_FILE);
    if log_path.exists() {
        fs::remove_file(log_path).expect("Failed to remove log file");
    }

    let project_toml_root = env::var("CARGO_MANIFEST_DIR").unwrap();
    // read the .config file at the project root
    let config_path = format!("{}/.config", project_toml_root);
    let mut build_env = parse_build_env(&config_path);

    if !build_env.bid.is_empty() {
        // BID=$ARCH/$BOARD, parse it
        // update the build_env with the parsed values
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
    } else {
        log(&format!(
            "BID environment variable not found, using ARCH and BOARD"
        ));
        if build_env.arch.is_empty() || build_env.board.is_empty() {
            log(&format!("ARCH or BOARD environment variable not found"));
            panic!(
                "ARCH or BOARD environment variable not found, please check the log file({}) for more details",
                BUILD_LOG_FILE
            );
        }
    }

    let arch = build_env.arch;
    let board = build_env.board;
    let bid = build_env.bid;
    let features = build_env.features;

    let pwd = env::current_dir().unwrap();
    log(&format!("Current directory: {}", pwd.display()));

    let target_path_str = format!("{}/src/platform/__board.rs", pwd.display());
    let target_path = Path::new(&target_path_str);
    let source_path_str = format!("{}/platform/{}/{}/board.rs", pwd.display(), arch, board);
    let source_path = Path::new(&source_path_str);

    log(&format!(
        "Building for ARCH={} BOARD={}, BID={}, FEATURES={}",
        arch, board, bid, features
    ));

    log(&format!(
        "Linking board.rs from {} to {}",
        source_path_str,
        target_path.display()
    ));

    if !source_path.exists() {
        log(&format!("Invalid board.rs path: {}, make sure the ARCH and BOARD environment variables are set correctly", source_path_str));
        panic!(
            "Invalid board.rs, please check the log file({}) for more details",
            BUILD_LOG_FILE
        );
    }

    // soft link the board.rs to __board.rs
    if target_path.exists() {
        fs::remove_file(target_path).expect("Failed to remove existing __board.rs");
    }
    std::os::unix::fs::symlink(source_path, target_path).expect("Failed to create symlink");
    log(&format!("Linking successful"));

    println!("cargo:rerun-if-env-changed=ARCH");
    println!("cargo:rerun-if-env-changed=BOARD");
    println!("cargo:rerun-if-env-changed=BID");
    println!("cargo:rerun-if-env-changed=FEATURES");
    println!("cargo:rerun-if-changed={}", source_path_str);
    println!("cargo:rerun-if-changed={}", target_path_str);
}
