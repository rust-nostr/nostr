// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::time::Duration;

use nostr::prelude::*;
use nostr_database::{DatabaseHelper, Order};
use tracing_subscriber::fmt::format::FmtSpan;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::fmt()
        .with_span_events(FmtSpan::CLOSE)
        .init();

    let secret_key =
        SecretKey::from_bech32("nsec1j4c6269y9w0q2er2xjw8sv2ehyrtfxq3jwgdlxj6qfn8z4gjsq5qfvfk99")
            .unwrap();
    let keys_a = Keys::new(secret_key);

    let secret_key =
        SecretKey::from_bech32("nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85")
            .unwrap();
    let keys_b = Keys::new(secret_key);

    let index = DatabaseHelper::unbounded();

    for i in 0..100_000 {
        let event = EventBuilder::text_note(format!("Event #{i}"), [])
            .to_event(&keys_a)
            .unwrap();
        index.index_event(&event).await;

        let event = EventBuilder::text_note(
            format!("Reply to event #{i}"),
            [Tag::event(event.id()), Tag::public_key(event.author())],
        )
        .to_event(&keys_b)
        .unwrap();
        index.index_event(&event).await;
    }

    for i in 0..1000 {
        let metadata = Metadata::new().name(format!("Name #{i}"));
        let event = EventBuilder::metadata(&metadata).to_event(&keys_a).unwrap();
        index.index_event(&event).await;
    }

    for i in 0..500_000 {
        let event = EventBuilder::new(
            Kind::Custom(123),
            "Custom with d tag",
            [Tag::identifier(format!("myid{i}"))],
        )
        .to_event(&keys_a)
        .unwrap();
        index.index_event(&event).await;
    }

    let ids = index
        .query(
            vec![Filter::new()
                .kinds(vec![Kind::Metadata, Kind::Custom(123), Kind::TextNote])
                .limit(20)
                //.kind(Kind::Custom(123))
                //.identifier("myid5000")
                .author(keys_a.public_key())],
            Order::Desc,
        )
        .await;
    println!("Got {} ids", ids.len());

    loop {
        tokio::time::sleep(Duration::from_secs(60)).await;
    }
}
