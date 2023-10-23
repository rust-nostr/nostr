// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::time::{Duration, Instant};

use nostr::{EventBuilder, Filter, Keys, Kind, Metadata, Tag};
use nostr_sdk_db::memory::MemoryDatabase;
use nostr_sdk_db::{DatabaseOptions, NostrDatabase};

#[tokio::main]
async fn main() {
    let keys = Keys::generate();
    let opts = DatabaseOptions::default();
    let database = MemoryDatabase::new(opts);

    for i in 0..50_000 {
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
    }

    for i in 0..10 {
        let metadata = Metadata::new().name(format!("Name #{i}"));
        let event = EventBuilder::set_metadata(metadata)
            .to_event(&keys)
            .unwrap();
        database.save_event(&event).await.unwrap();
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    let now = Instant::now();
    let events = database
        .query(vec![Filter::new()
            .kind(Kind::Metadata)
            .author(keys.public_key().to_string())])
        .await
        .unwrap();
    println!("{events:?}");
    println!("Time: {} ns", now.elapsed().as_nanos());
}
