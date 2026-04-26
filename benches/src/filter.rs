// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::hint::black_box;
use std::str::FromStr;

use criterion::Criterion;
use nostr::prelude::*;

fn filter_match_event(c: &mut Criterion) {
    // Event
    let event =
            Event::new(
                EventId::from_hex("70b10f70c1318967eddf12527799411b1a9780ad9c43858f5e5fcd45486a13a5")
                .unwrap(),
                PublicKey::from_hex("379e863e8357163b5bce5d2688dc4f1dcc2d505222fb8d74db600f30535dfdfe")
                .unwrap(),
                Timestamp::from(1612809991),
                Kind::TextNote,
                [
                    Tag::public_key(PublicKey::from_hex("b2d670de53b27691c0c3400225b65c35a26d06093bcc41f48ffc71e0907f9d4a").unwrap()),
                    Tag::public_key(PublicKey::from_hex("379e863e8357163b5bce5d2688dc4f1dcc2d505222fb8d74db600f30535dfdfe").unwrap()),
                    Tag::event(EventId::from_hex("7469af3be8c8e06e1b50ef1caceba30392ddc0b6614507398b7d7daa4c218e96").unwrap()),
                    Tag::from_standardized(TagStandard::Kind { kind: Kind::TextNote, uppercase: false }),
                ],
                "#JoininBox is a minimalistic, security focused Linux environment for #JoinMarket with a terminal based graphical menu.\n\nnostr:npub14tq8m9ggnnn2muytj9tdg0q6f26ef3snpd7ukyhvrxgq33vpnghs8shy62 👍🧡\n\nhttps://www.nobsbitcoin.com/joininbox-v0-8-0/",
                Signature::from_str("273a9cd5d11455590f4359500bccb7a89428262b96b3ea87a756b770964472f8c3e87f5d5e64d8d2e859a71462a3f477b554565c4f2f326cb01dd7620db71502").unwrap(),
            );

    // Filter
    let pk =
        PublicKey::from_hex("b2d670de53b27691c0c3400225b65c35a26d06093bcc41f48ffc71e0907f9d4a")
            .unwrap();
    let filter = Filter::new()
        .pubkey(pk)
        .search("linux")
        .kind(Kind::TextNote);

    c.bench_function("filter_match_event", |bh| {
        bh.iter(|| {
            black_box(filter.match_event(&event, MatchEventOptions::new()));
        })
    });
}

pub fn benches(c: &mut Criterion) {
    filter_match_event(c);
}
