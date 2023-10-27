// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

// use std::time::{Duration, Instant};

use nostr::prelude::*;
use nostr_sdk_db::NostrDatabase;
use nostr_sdk_rocksdb::RocksDatabase;
use tracing_subscriber::fmt::format::FmtSpan;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::fmt()
        .with_span_events(FmtSpan::CLOSE)
        .init();

    let secret_key =
        SecretKey::from_bech32("nsec1j4c6269y9w0q2er2xjw8sv2ehyrtfxq3jwgdlxj6qfn8z4gjsq5qfvfk99")
            .unwrap();
    let keys = Keys::new(secret_key);
    let database = RocksDatabase::new("./db/rocksdb").unwrap();

    /* for i in 0..50_000 {
        let event = EventBuilder::new_text_note(format!("Event #{i}"), &[])
            .to_event(&keys)
            .unwrap();
        database.save_event(&event).await.unwrap();

        let event = EventBuilder::new_text_note(
            format!("Reply to event #{i}"),
            &[
                Tag::Event(event.id, None, None),
                Tag::PubKey(event.pubkey, None),
            ],
        )
        .to_event(&keys)
        .unwrap();
       database.save_event(&event).await.unwrap();
       println!("{}", event.id);
    }

    for i in 0..10 {
        let metadata = Metadata::new().name(format!("Name #{i}"));
        let event = EventBuilder::set_metadata(metadata)
            .to_event(&keys)
            .unwrap();
        database.save_event(&event).await.unwrap();
        tokio::time::sleep(Duration::from_secs(1)).await;
    } */

    /* let event_id =
        EventId::from_hex("b02c1c57a7c5b0e10245df8c26b429ad1a2cbf91d7cada3ecdb524b7e1d984b6")
            .unwrap();
    let event = database.event_by_id(event_id).await.unwrap();
    println!("{event:?}"); */

    let events = database
        .query(vec![Filter::new()
            .kind(Kind::Metadata)
            .author(keys.public_key().to_string())])
        .await
        .unwrap();
    println!("Got {} events", events.len());
}
