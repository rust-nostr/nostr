// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::hint::black_box;

use criterion::Criterion;
use nostr::prelude::*;
use nostr_database::flatbuffers::FlatBufferDecodeBorrowed;
use nostr_database::{FlatBufferBuilder, FlatBufferEncode};

pub fn decode_flatbuf_event_borrow(c: &mut Criterion) {
    let json = r#"{
              "content": "+",
              "created_at": 1716508454,
              "id": "3e9e9c2fbf263590860a9c60a7de6b0d166230a5a15aa8dcdb70f537cec9807a",
              "kind": 7,
              "pubkey": "3bbddb5c7233ad993b41cb639e63122120f391b8580a9b83aae33c648230e0a3",
              "sig": "3f2ba6d713e4851500b81de2d2ef44b72f1eff061898bf8488e74f7e4ed141b0dadab4c3a9c6b237f3a6db83171bd41eafd7ab973f6fb067a4305e95abeadeee",
              "tags": [
                [
                  "e",
                  "e1e786c60ed884b6e784712aaf70e63b848b7403ef651b52b701d87739ea1808",
                  "",
                  "",
                  "04c915daefee38317fa734444acee390a8269fe5810b2241e5e6dd343dfbecc9"
                ],
                [
                  "p",
                  "04c915daefee38317fa734444acee390a8269fe5810b2241e5e6dd343dfbecc9"
                ]
              ]
            }"#;
    let event = Event::from_json(json).unwrap();

    let mut fbb = FlatBufferBuilder::new();
    let bytes = event.encode(&mut fbb);

    c.bench_function("decode_flatbuf_event_borrow", |bh| {
        bh.iter(|| {
            black_box(EventBorrow::decode(bytes)).unwrap();
        })
    });
}

pub fn benches(c: &mut Criterion) {
    decode_flatbuf_event_borrow(c);
}
