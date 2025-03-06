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

fn main() {
    // cleanup the log file
    let log_path = Path::new(BUILD_LOG_FILE);
    if log_path.exists() {
        fs::remove_file(log_path).expect("Failed to remove log file");
    }

    let arch = env::var("ARCH").expect("ARCH environment variable not set");
    let board = env::var("BOARD").expect("BOARD environment variable not set");
    let features = env::var("FEATURES").expect("FEATURES environment variable not set");
    let rustc_target = env::var("RUSTC_TARGET").expect("RUSTC_TARGET environment variable not set");

    let pwd = env::current_dir().unwrap();
    log(&format!("Current directory: {}", pwd.display()));

    let target_path_str = format!("{}/src/platform/__board.rs", pwd.display());
    let target_path = Path::new(&target_path_str);
    let source_path_str = format!("{}/platform/{}/{}/board.rs", pwd.display(), arch, board);
    let source_path = Path::new(&source_path_str);

    log(&format!(
        "Building for ARCH={} BOARD={}, FEATURES={}, RUSTC_TARGET={}",
        arch, board, features, rustc_target
    ));

    log(&format!(
        "Copying {} to {}",
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

    // copy the board.rs file to __board.rs
    let r = fs::copy(source_path, target_path);
    if let Err(e) = r {
        log(&format!("Failed to copy board.rs: {}", e));
        panic!(
            "Failed to copy board.rs, please check the log file({}) for more details",
            BUILD_LOG_FILE
        );
    }

    println!("cargo:rerun-if-env-changed=ARCH");
    println!("cargo:rerun-if-env-changed=BOARD");
    println!("cargo:rerun-if-env-changed=FEATURES");
    println!("cargo:rerun-if-changed={}", source_path_str);
    println!("cargo:rerun-if-changed={}", target_path_str);
}
