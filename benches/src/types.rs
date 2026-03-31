// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::hint::black_box;

use criterion::Criterion;
use nostr::types::{RelayUrl, Timestamp};

const LOCAL_URL: &str = "ws://127.0.0.1:7777";
const CLEARNET_URL: &str = "wss://relay.damus.io";
const ONION_URL: &str = "ws://oxtrdevav64z64yb7x6rjg4ntzqjhedm5b5zjqulugknhzr46ny2qbad.onion";

fn timestamp_to_human_datetime(c: &mut Criterion) {
    let timestamp = Timestamp::from(1682060685);

    c.bench_function("timestamp_to_human_datetime", |bh| {
        bh.iter(|| {
            black_box(timestamp.to_human_datetime());
        })
    });
}

fn parse_local_relay_url(c: &mut Criterion) {
    c.bench_function("parse_local_relay_url", |bh| {
        bh.iter(|| {
            black_box(RelayUrl::parse(LOCAL_URL)).unwrap();
        })
    });
}

fn parse_clearnet_relay_url(c: &mut Criterion) {
    c.bench_function("parse_clearnet_relay_url", |bh| {
        bh.iter(|| {
            black_box(RelayUrl::parse(CLEARNET_URL)).unwrap();
        })
    });
}

fn parse_onion_relay_url(c: &mut Criterion) {
    c.bench_function("parse_onion_relay_url", |bh| {
        bh.iter(|| {
            black_box(RelayUrl::parse(ONION_URL)).unwrap();
        })
    });
}

pub fn benches(c: &mut Criterion) {
    timestamp_to_human_datetime(c);

    parse_local_relay_url(c);
    parse_clearnet_relay_url(c);
    parse_onion_relay_url(c);
}
