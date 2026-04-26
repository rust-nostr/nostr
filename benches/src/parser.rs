// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::hint::black_box;

use criterion::Criterion;
use nostr::prelude::*;

const PARSER: NostrParser = NostrParser::new();

fn parse_text_with_nostr_uris(c: &mut Criterion) {
    let text: &str = "I have never been very active in discussions but working on rust-nostr (at the time called nostr-rs-sdk) since September 2022 🦀 \n\nIf I remember correctly there were also nostr:nprofile1qqsqfyvdlsmvj0nakmxq6c8n0c2j9uwrddjd8a95ynzn9479jhlth3gpvemhxue69uhkv6tvw3jhytnwdaehgu3wwa5kuef0dec82c33w94xwcmdd3cxketedsux6ertwecrgues0pk8xdrew33h27pkd4unvvpkw3nkv7pe0p68gat58ycrw6ps0fenwdnvva48w0mzwfhkzerrv9ehg0t5wf6k2qgnwaehxw309ac82unsd3jhqct89ejhxtcpz4mhxue69uhhyetvv9ujuerpd46hxtnfduhsh8njvk and nostr:nprofile1qqswuyd9ml6qcxd92h6pleptfrcqucvvjy39vg4wx7mv9wm8kakyujgpypmhxue69uhkx6r0wf6hxtndd94k2erfd3nk2u3wvdhk6w35xs6z7qgwwaehxw309ahx7uewd3hkctcpypmhxue69uhkummnw3ezuetfde6kuer6wasku7nfvuh8xurpvdjj7a0nq40";

    c.bench_function("parse_text_with_nostr_uris", |bh| {
        bh.iter(|| {
            black_box(PARSER.parse(text).collect::<Vec<_>>());
        })
    });
}

fn parse_text_with_urls(c: &mut Criterion) {
    let text: &str = "I've uses both the book and rustlings: https://github.com/rust-lang/rustlings/\n\nThere is also the \"Rust by example\" book: https://doc.rust-lang.org/rust-by-example/\n\nWhile you read the book, try to make projects from scratch (not just simple ones). At the end, writing code is the best way to learn it.";

    c.bench_function("parse_text_with_urls", |bh| {
        bh.iter(|| {
            black_box(PARSER.parse(text).collect::<Vec<_>>());
        })
    });
}

fn parse_text_with_hashtags(c: &mut Criterion) {
    let text: &str = "Hey #rust-nostr fans, can you enlighten me please:\nWhen I am calculating my Web of Trust I do the following:\n0. Create client with outbox model enabled\n1. Get my follows, mutes, reports in one fetch call\n2. Get follows, mutes, reports of my follows in another fetch call, using an authors filter that has all follows in it\n3. Calculate scores with my weights locally\n\nQuestion:\nWhy did step 2. take hours to complete?\n\nIt seems like it's trying to connect to loads of relays.\nMy guess is either I am doing sth horribly wrong or there is no smart relay set calculation for filters in the pool.\n\nIn ndk this calculation takes under 10 seconds to complete, even without any caching. It will first look at the filters and calculate a relay set that has all authors in it then does the fetching.\n\n#asknostr #rust";

    c.bench_function("parse_text_with_hashtags", |bh| {
        bh.iter(|| {
            black_box(PARSER.parse(text).collect::<Vec<_>>());
        })
    });
}

pub fn benches(c: &mut Criterion) {
    parse_text_with_nostr_uris(c);
    parse_text_with_urls(c);
    parse_text_with_hashtags(c);
}
