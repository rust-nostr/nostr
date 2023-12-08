// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

#[cfg(feature = "std")]
use std::process::Command;

fn main() {
    #[cfg(feature = "std")]
    if let Ok(output) = Command::new("git").args(["rev-parse", "HEAD"]).output() {
        if let Ok(git_hash) = String::from_utf8(output.stdout) {
            println!("cargo:rustc-env=GIT_HASH={git_hash}");
        }
    }
}
