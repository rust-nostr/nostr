// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

#[cfg(bench)]
criterion::criterion_main!(benches::benches);

#[cfg(not(bench))]
fn main() {
    println!("No benchmarks without cfg=bench");
    println!("Run with RUSTFLAGS='--cfg=bench -Awarnings'");
}
