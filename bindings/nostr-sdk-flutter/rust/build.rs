// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::process::{Command, Stdio};

use lib_flutter_rust_bridge_codegen::codegen;
use lib_flutter_rust_bridge_codegen::codegen::Config;

fn main() {
    println!("cargo:rerun-if-changed=src/api");

    if is_flutter_installed() {
        // Execute code generator with auto-detected config
        codegen::generate(
            Config::from_config_file("../flutter_rust_bridge.yaml")
                .unwrap()
                .unwrap(),
            Default::default(),
        )
        .unwrap();
    } else {
        eprintln!("Warning: flutter not installed.");
    }
}

fn is_flutter_installed() -> bool {
    let output = Command::new("flutter")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    matches!(output, Ok(status) if status.success())
}
