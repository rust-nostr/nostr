// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::hint::black_box;
use std::str::FromStr;

use criterion::Criterion;
use nostr::prelude::*;

fn parse_coordinate(c: &mut Criterion) {
    let coordinate: &str =
        "30023:aa4fc8665f5696e33db7e1a572e3b0f5b3d615837b0f362dcb1c8068b098c7b4:ipsum";

    c.bench_function("parse_coordinate", |bh| {
        bh.iter(|| black_box(Coordinate::parse(coordinate)).unwrap())
    });
}

fn to_bech32_nevent(c: &mut Criterion) {
    let event_id =
        EventId::from_hex("d94a3f4dd87b9a3b0bed183b32e916fa29c8020107845d1752d72697fe5309a5")
            .unwrap();
    let public_key =
        PublicKey::from_str("32e1827635450ebb3c5a7d12c1f8e7b2b514439ac10a67eef3d9fd9c5c68e245")
            .unwrap();
    let relays = [
        RelayUrl::parse("wss://r.x.com").unwrap(),
        RelayUrl::parse("wss://djbas.sadkb.com").unwrap(),
    ];
    let nip19_event = Nip19Event::new(event_id).author(public_key).relays(relays);

    c.bench_function("to_bech32_nevent", |bh| {
        bh.iter(|| {
            black_box(nip19_event.to_bech32()).unwrap();
        })
    });
}

pub fn benches(c: &mut Criterion) {
    parse_coordinate(c);
    to_bech32_nevent(c);
}
