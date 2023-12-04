// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::time::Duration;

use nostr::prelude::*;
use nostr_database::NostrDatabase;
use nostr_rocksdb::RocksDatabase;
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

    let database = RocksDatabase::open("./db/rocksdb").await.unwrap();

    println!(
        "Events stored: {}",
        database.count(vec![Filter::new()]).await.unwrap()
    );

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
    }

    for i in 0..500_000 {
        let event = EventBuilder::new(
            Kind::Custom(123),
            "Custom with d tag",
            &[Tag::Identifier(format!("myid{i}"))],
        )
        .to_event(&keys_a)
        .unwrap();
        database.save_event(&event).await.unwrap();
    } */

    /* let event_id = EventId::all_zeros();
    database.event_id_seen(event_id, Some(Url::parse("wss://relay.damus.io").unwrap())).await.unwrap();
    database.event_id_seen(event_id, Some(Url::parse("wss://relay.nostr.info").unwrap())).await.unwrap();
    database.event_id_seen(event_id, Some(Url::parse("wss://relay.damus.io").unwrap())).await.unwrap();

    let relays = database.event_seen_on_relays(event_id).await.unwrap();
    println!("Seen on: {relays:?}"); */

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

    loop {
        tokio::time::sleep(Duration::from_secs(30)).await
    }
}
