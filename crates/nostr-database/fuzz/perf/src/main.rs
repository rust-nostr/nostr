use std::collections::BTreeSet;
use std::fs::File;
use std::io::{BufRead, BufReader};

use nostr_database::nostr::RelayMessage;
use nostr_database::{DatabaseIndexes, RawEvent};
use tracing_subscriber::fmt::format::FmtSpan;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_span_events(FmtSpan::CLOSE)
        .init();

    // Load events
    // Open JSON file
    let file = File::open("./many-events.json").unwrap();
    let metadata = file.metadata().unwrap();
    let reader = BufReader::new(file);

    println!("Size: {}", metadata.len());

    // Deserialize events
    let mut events: BTreeSet<RawEvent> = BTreeSet::new();

    for line in reader.lines() {
        match line {
            Ok(line_content) => {
                if let Ok(RelayMessage::Event { event, .. }) = serde_json::from_str(&line_content) {
                    events.insert(event.as_ref().into());
                }
            }
            Err(e) => eprintln!("Error reading line: {}", e),
        }
    }

    // Indexes
    let indexes = DatabaseIndexes::new();

    // Bulk load
    println!("Indexing {} events", events.len());
    indexes.bulk_index(events).await;
}
