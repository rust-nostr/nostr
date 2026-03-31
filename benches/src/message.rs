// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::hint::black_box;

use criterion::Criterion;
use nostr::prelude::*;

fn parse_machine_readable_prefix(c: &mut Criterion) {
    c.bench_function("parse_machine_readable_prefix", |bh| {
        bh.iter(|| {
            black_box(MachineReadablePrefix::parse(
                "blocked: you are banned from posting here",
            ))
            .unwrap();
        })
    });
}

fn parse_ok_relay_message(c: &mut Criterion) {
    let json: &str = r#"["OK", "70b10f70c1318967eddf12527799411b1a9780ad9c43858f5e5fcd45486a13a5", true, "pow: difficulty 25>=24"]"#;
    c.bench_function("parse_ok_relay_message", |bh| {
        bh.iter(|| {
            black_box(RelayMessage::from_json(&json)).unwrap();
        })
    });
}

fn parse_event_relay_message(c: &mut Criterion) {
    let json: &str = r#"["EVENT", "random_string", {"id":"70b10f70c1318967eddf12527799411b1a9780ad9c43858f5e5fcd45486a13a5","pubkey":"379e863e8357163b5bce5d2688dc4f1dcc2d505222fb8d74db600f30535dfdfe","created_at":1612809991,"kind":1,"tags":[],"content":"test","sig":"273a9cd5d11455590f4359500bccb7a89428262b96b3ea87a756b770964472f8c3e87f5d5e64d8d2e859a71462a3f477b554565c4f2f326cb01dd7620db71502"}]"#;
    c.bench_function("parse_event_relay_message", |bh| {
        bh.iter(|| {
            black_box(RelayMessage::from_json(&json)).unwrap();
        })
    });
}

pub fn benches(c: &mut Criterion) {
    parse_machine_readable_prefix(c);
    parse_ok_relay_message(c);
    parse_event_relay_message(c);
}
