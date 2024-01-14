use std::collections::BTreeSet;

use nostr_database::nostr::RelayMessage;
use nostr_database::{DatabaseIndexes, RawEvent};
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};
use tracing_subscriber::fmt::format::FmtSpan;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_span_events(FmtSpan::CLOSE)
        .init();

    // Load events
    // Open JSON file
    let file = File::open("./many-events.json").await.unwrap();
    let metadata = file.metadata().await.unwrap();
    let reader = BufReader::new(file);

    println!("Size: {}", metadata.len());

    // Deserialize events
    let mut events: BTreeSet<RawEvent> = BTreeSet::new();

    let mut lines = reader.lines();
    while let Ok(Some(line)) = lines.next_line().await {
        if let Ok(RelayMessage::Event { event, .. }) = serde_json::from_str(&line) {
            events.insert(event.as_ref().into());
        }
    }

    // Indexes
    let indexes = DatabaseIndexes::new();

    // Bulk load
    println!("Indexing {} events", events.len());
    indexes.bulk_index(events).await;
}
