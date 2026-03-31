// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::hint::black_box;

use criterion::Criterion;
use nostr::prelude::*;

const ID: &str = "2be17aa3031bdcb006f0fce80c146dea9c1c0268b0af2398bb673365c6444d45";

fn deserialize_event(c: &mut Criterion) {
    let json = r#"{"content":"uRuvYr585B80L6rSJiHocw==?iv=oh6LVqdsYYol3JfFnXTbPA==","created_at":1640839235,"id":"2be17aa3031bdcb006f0fce80c146dea9c1c0268b0af2398bb673365c6444d45","kind":4,"pubkey":"f86c44a2de95d9149b51c6a29afeabba264c18e2fa7c49de93424a0c56947785","sig":"a5d9290ef9659083c490b303eb7ee41356d8778ff19f2f91776c8dc4443388a64ffcf336e61af4c25c05ac3ae952d1ced889ed655b67790891222aaa15b99fdd","tags":[["p","13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"]]}"#;
    c.bench_function("deserialize_event", |bh| {
        bh.iter(|| black_box(Event::from_json(json)).unwrap())
    });
}

fn serialize_event(c: &mut Criterion) {
    let json = r#"{"content":"uRuvYr585B80L6rSJiHocw==?iv=oh6LVqdsYYol3JfFnXTbPA==","created_at":1640839235,"id":"2be17aa3031bdcb006f0fce80c146dea9c1c0268b0af2398bb673365c6444d45","kind":4,"pubkey":"f86c44a2de95d9149b51c6a29afeabba264c18e2fa7c49de93424a0c56947785","sig":"a5d9290ef9659083c490b303eb7ee41356d8778ff19f2f91776c8dc4443388a64ffcf336e61af4c25c05ac3ae952d1ced889ed655b67790891222aaa15b99fdd","tags":[["p","13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"]]}"#;
    let event = Event::from_json(json).unwrap();
    c.bench_function("serialize_event", |bh| {
        bh.iter(|| black_box(event.as_json()))
    });
}

fn event_check_pow(c: &mut Criterion) {
    let json = r#"{"id":"000006d11924b38e55275637c6401965c54f9ae05ffed89bce0edc1720984656","pubkey":"385c3a6ec0b9d57a4330dbd6284989be5bd00e41c535f9ca39b6ae7c521b81cd","created_at":1759497131,"kind":1,"tags":[["nonce","1180727","20"]],"content":"This is a Nostr message with embedded proof-of-work","sig":"0b216dfa714db2f146f9fa7cf20954c3fd5a2dabf69cd30ab58cf142a7ebe0fd3f4bc8e9c261245dabc0be8f942ec15d3fe3ce4dcbe81df01ceb4ced91739f52"}"#;
    let event = Event::from_json(json).unwrap();
    c.bench_function("event_check_pow", |bh| {
        bh.iter(|| black_box(event.check_pow(16)))
    });
}

fn verify_event_id(c: &mut Criterion) {
    let json = r#"{"content":"uRuvYr585B80L6rSJiHocw==?iv=oh6LVqdsYYol3JfFnXTbPA==","created_at":1640839235,"id":"2be17aa3031bdcb006f0fce80c146dea9c1c0268b0af2398bb673365c6444d45","kind":4,"pubkey":"f86c44a2de95d9149b51c6a29afeabba264c18e2fa7c49de93424a0c56947785","sig":"a5d9290ef9659083c490b303eb7ee41356d8778ff19f2f91776c8dc4443388a64ffcf336e61af4c25c05ac3ae952d1ced889ed655b67790891222aaa15b99fdd","tags":[["p","13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"]]}"#;
    let event = Event::from_json(json).unwrap();
    c.bench_function("verify_event_id", |bh| {
        bh.iter(|| black_box(event.verify_id()))
    });
}

fn verify_event_sig(c: &mut Criterion) {
    let json = r#"{"content":"uRuvYr585B80L6rSJiHocw==?iv=oh6LVqdsYYol3JfFnXTbPA==","created_at":1640839235,"id":"2be17aa3031bdcb006f0fce80c146dea9c1c0268b0af2398bb673365c6444d45","kind":4,"pubkey":"f86c44a2de95d9149b51c6a29afeabba264c18e2fa7c49de93424a0c56947785","sig":"a5d9290ef9659083c490b303eb7ee41356d8778ff19f2f91776c8dc4443388a64ffcf336e61af4c25c05ac3ae952d1ced889ed655b67790891222aaa15b99fdd","tags":[["p","13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"]]}"#;
    let event = Event::from_json(json).unwrap();
    c.bench_function("verify_event_sig", |bh| {
        bh.iter(|| black_box(event.verify_signature()))
    });
}

fn parse_event_id_from_hex(c: &mut Criterion) {
    c.bench_function("parse_event_id_from_hex", |bh| {
        bh.iter(|| black_box(EventId::from_hex(ID)).unwrap())
    });
}

fn parse_ephemeral_kind(c: &mut Criterion) {
    c.bench_function("parse_ephemeral_kind", |bh| {
        bh.iter(|| black_box(Kind::from(29_999)))
    });
}

fn parse_kind(c: &mut Criterion) {
    c.bench_function("parse_kind", |bh| bh.iter(|| black_box(Kind::from(0))));
}

fn builder_to_event(c: &mut Criterion) {
    let keys = Keys::generate();
    c.bench_function("builder_to_event", |bh| {
        bh.iter(|| black_box(EventBuilder::text_note("hello").sign_with_keys(&keys)).unwrap())
    });
}

pub fn benches(c: &mut Criterion) {
    deserialize_event(c);
    serialize_event(c);
    event_check_pow(c);
    verify_event_id(c);
    verify_event_sig(c);

    parse_event_id_from_hex(c);
    parse_ephemeral_kind(c);
    parse_kind(c);
    builder_to_event(c);
}
