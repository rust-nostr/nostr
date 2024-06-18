// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

fn main() {
    // 0x1E84800 bytes = 32MiB
    println!("cargo:rustc-link-arg=-zstack-size=0x1E84800");
}
