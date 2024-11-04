// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::env;
use std::process::Command;

use cbindgen::{Config, Error, Language};

fn main() {
    if let Ok(output) = Command::new("git").args(["rev-parse", "HEAD"]).output() {
        if let Ok(git_hash) = String::from_utf8(output.stdout) {
            println!("cargo:rerun-if-changed={git_hash}");
            println!("cargo:rustc-env=GIT_HASH={git_hash}");
        }
    }

    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let config: Config = config();
    cbindgen::generate_with_config(&crate_dir, config).map_or_else(
        |error| match error {
            Error::ParseSyntaxError { .. } => {}
            e => panic!("{:?}", e),
        },
        |bindings| {
            bindings.write_to_file("include/nostr_sdk.h");
        },
    );
}

fn config() -> Config {
    let mut config: Config = Config::default();
    config.language = Language::C;
    config.cpp_compat = true;
    config.documentation = true;
    config
}
