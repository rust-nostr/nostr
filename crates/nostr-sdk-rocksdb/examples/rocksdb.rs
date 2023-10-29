// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::time::Duration;

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
    let keys_a = Keys::new(secret_key);
    println!("Pubkey A: {}", keys_a.public_key());

    let secret_key =
        SecretKey::from_bech32("nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85")
            .unwrap();
    let keys_b = Keys::new(secret_key);
    println!("Pubkey B: {}", keys_b.public_key());

    let database = RocksDatabase::new("./db/rocksdb").unwrap();
    database.build_indexes().await.unwrap();

    /* for i in 0..100_000 {
        let event = EventBuilder::new_text_note(format!("Event #{i}"), &[])
            .to_event(&keys_a)
            .unwrap();
        database.save_event(&event).await.unwrap();

        let event = EventBuilder::new_text_note(
            format!("Reply to event #{i}"),
            &[
                Tag::Event(event.id, None, None),
                Tag::PubKey(event.pubkey, None),
            ],
        )
        .to_event(&keys_b)
        .unwrap();
        database.save_event(&event).await.unwrap();
    }

    for i in 0..10 {
        let metadata = Metadata::new().name(format!("Name #{i}"));
        let event = EventBuilder::set_metadata(metadata)
            .to_event(&keys_a)
            .unwrap();
        database.save_event(&event).await.unwrap();
        tokio::time::sleep(Duration::from_secs(1)).await;
    } */

    /*     let event = EventBuilder::new(Kind::Custom(123), "Custom with d tag", &[Tag::Identifier(String::from("myid"))])
        .to_event(&keys)
        .unwrap();
    database.save_event(&event).await.unwrap(); */

    /* let event_id =
        EventId::from_hex("b02c1c57a7c5b0e10245df8c26b429ad1a2cbf91d7cada3ecdb524b7e1d984b6")
            .unwrap();
    let event = database.event_by_id(event_id).await.unwrap();
    println!("{event:?}"); */

    let events = database
        .query(vec![Filter::new()
            .kind(Kind::Metadata)
            //.limit(1)
            //.kind(Kind::Custom(123))
            //.identifier("myid")
            .author(keys_a.public_key())])
        .await
        .unwrap();
    //println!("{events:?}");
    println!("Got {} events", events.len());

    loop {
        tokio::time::sleep(Duration::from_secs(30)).await
    }
}
