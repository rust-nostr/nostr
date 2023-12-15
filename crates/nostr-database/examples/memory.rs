// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use nostr::prelude::*;
use nostr::{EventBuilder, Filter, Keys, Kind, Metadata, Tag};
use nostr_database::memory::MemoryDatabase;
use nostr_database::{DatabaseOptions, NostrDatabase};
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

    let mut opts = DatabaseOptions::default();
    opts.events = true;
    let database = MemoryDatabase::new(opts);

    for i in 0..100_000 {
        let event = EventBuilder::new_text_note(format!("Event #{i}"), [])
            .to_event(&keys_a)
            .unwrap();
        database.save_event(&event).await.unwrap();

        let event = EventBuilder::new_text_note(
            format!("Reply to event #{i}"),
            [Tag::event(event.id), Tag::public_key(event.pubkey)],
        )
        .to_event(&keys_b)
        .unwrap();
        database.save_event(&event).await.unwrap();
    }

    for i in 0..10 {
        let metadata = Metadata::new().name(format!("Name #{i}"));
        let event = EventBuilder::set_metadata(&metadata)
            .to_event(&keys_a)
            .unwrap();
        database.save_event(&event).await.unwrap();
    }

    for i in 0..500_000 {
        let event = EventBuilder::new(
            Kind::Custom(123),
            "Custom with d tag",
            [Tag::Identifier(format!("myid{i}"))],
        )
        .to_event(&keys_a)
        .unwrap();
        database.save_event(&event).await.unwrap();
    }

    let events = database
        .query(vec![Filter::new()
            .kinds(vec![Kind::Metadata, Kind::Custom(123), Kind::TextNote])
            .limit(20)
            //.kind(Kind::Custom(123))
            //.identifier("myid5000")
            .author(keys_a.public_key())])
        .await
        .unwrap();
    println!("Got {} events", events.len());

    loop {}
}
