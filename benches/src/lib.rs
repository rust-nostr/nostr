// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

#![cfg(bench)]

mod database;
mod event;
mod filter;
mod keys;
mod message;
mod nips;
mod parser;
mod relay;
mod tags;
mod types;

criterion::criterion_group!(
    benches,
    nips::benches,
    keys::benches,
    tags::benches,
    event::benches,
    relay::benches,
    types::benches,
    filter::benches,
    message::benches,
    parser::benches,
    database::benches,
);
