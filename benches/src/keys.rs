// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::hint::black_box;

use criterion::Criterion;
use nostr::prelude::*;

const NIP21_URI: &str = "nostr:npub14f8usejl26twx0dhuxjh9cas7keav9vr0v8nvtwtrjqx3vycc76qqh9nsy";
const HEX: &str = "aa4fc8665f5696e33db7e1a572e3b0f5b3d615837b0f362dcb1c8068b098c7b4";
const BECH32: &str = "npub14f8usejl26twx0dhuxjh9cas7keav9vr0v8nvtwtrjqx3vycc76qqh9nsy";

fn parse_public_key_nip21_uri(c: &mut Criterion) {
    c.bench_function("parse_public_key_nip21_uri", |bh| {
        bh.iter(|| {
            black_box(PublicKey::parse(NIP21_URI)).unwrap();
        })
    });
}

fn parse_public_key_hex(c: &mut Criterion) {
    c.bench_function("parse_public_key_hex", |bh| {
        bh.iter(|| {
            black_box(PublicKey::parse(HEX)).unwrap();
        })
    });
}

fn parse_public_key_bech32(c: &mut Criterion) {
    c.bench_function("parse_public_key_bech32", |bh| {
        bh.iter(|| {
            black_box(PublicKey::parse(BECH32)).unwrap();
        })
    });
}

fn public_key_from_bech32(c: &mut Criterion) {
    c.bench_function("public_key_from_bech32", |bh| {
        bh.iter(|| {
            black_box(PublicKey::from_bech32(BECH32)).unwrap();
        })
    });
}

fn public_key_to_bech32(c: &mut Criterion) {
    let public_key = PublicKey::from_hex(HEX).unwrap();
    c.bench_function("public_key_to_bech32", |bh| {
        bh.iter(|| {
            black_box(public_key.to_bech32()).unwrap();
        })
    });
}

fn public_key_from_hex(c: &mut Criterion) {
    c.bench_function("public_key_from_hex", |bh| {
        bh.iter(|| {
            black_box(PublicKey::from_hex(HEX)).unwrap();
        })
    });
}

fn public_key_to_hex(c: &mut Criterion) {
    let public_key = PublicKey::from_hex(HEX).unwrap();
    c.bench_function("public_key_to_hex", |bh| {
        bh.iter(|| {
            black_box(public_key.to_hex());
        })
    });
}

pub fn benches(c: &mut Criterion) {
    parse_public_key_nip21_uri(c);
    parse_public_key_hex(c);
    parse_public_key_bech32(c);
    public_key_from_bech32(c);
    public_key_to_bech32(c);
    public_key_from_hex(c);
    public_key_to_hex(c);
}
