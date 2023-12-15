use nostr_database::nostr::Event;
use nostr_database::DatabaseIndexes;
use tracing_subscriber::fmt::format::FmtSpan;

mod constants;

use self::constants::EVENTS;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::fmt()
        .with_span_events(FmtSpan::CLOSE)
        .init();

    // Load events
    let events: Vec<Event> = serde_json::from_str(EVENTS).unwrap();
    let len = events.len();
    let events = events.into_iter().map(|e| e.into()).collect();

    // Indexes
    let indexes = DatabaseIndexes::new();

    // Bulk load
    println!("Indexing {len} events");
    indexes.bulk_index(events).await;
}
