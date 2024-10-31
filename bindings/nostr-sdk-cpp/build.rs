// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::fs;

use glob::glob;

fn main() {
    let mut files: Vec<String> = Vec::new();

    // Recursively find all .rs files in src/ directory
    for entry in glob("src/**/*.rs").expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => {
                // Check if file contains `#[cxx::bridge` (must be WITHOUT last bracket!)
                if let Ok(content) = fs::read_to_string(&path) {
                    if content.contains("#[cxx::bridge") {
                        files.push(path.to_string_lossy().to_string());
                    }
                }
            }
            Err(e) => println!("Error processing file: {:?}", e),
        }
    }

    // Assert
    assert!(!files.is_empty(), "No source file is provided.");

    // Build
    cxx_build::bridges(&files)
        .std("c++11")
        .compile("nostr_sdk_cpp");

    // Rerun if changed conditions
    println!("cargo:rerun-if-changed=src/");
}
