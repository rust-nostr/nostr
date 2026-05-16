// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use nostr::event::{EventBuilder, FinalizeEvent, Tag};
use nostr::filter::Filter;
use nostr::key::Keys;
use nostr::util::BoxedFuture;
use nostr_memory::MemoryDatabase;
use nostr_relay_builder::LocalRelay;
use nostr_relay_builder::builder::{QueryPolicy, QueryPolicyResult};
use nostr_sdk::client::Client;

const UPDATE_TAG: &str = "updated";

#[derive(Debug)]
struct UpdateFilterPlugin;

impl QueryPolicy for UpdateFilterPlugin {
    fn admit_query<'a>(
        &'a self,
        query: &'a mut Filter,
        _addr: &'a std::net::SocketAddr,
    ) -> BoxedFuture<'a, QueryPolicyResult> {
        Box::pin(async move {
            *query = query.clone().hashtag(UPDATE_TAG);
            QueryPolicyResult::Accept
        })
    }
}

#[tokio::test]
async fn update_filter() {
    let relay = LocalRelay::builder()
        .database(MemoryDatabase::unbounded())
        .query_policy(UpdateFilterPlugin)
        .build();
    relay.run().await.unwrap();

    let keys = Keys::generate();
    let client = Client::default();

    client
        .add_relay(relay.url().await)
        .and_connect()
        .await
        .unwrap();

    // Event with our target tag
    let event = EventBuilder::text_note(":)")
        .tag(Tag::hashtag(UPDATE_TAG))
        .finalize(&keys)
        .unwrap();
    client.send_event(&event).await.unwrap();

    // This event has a random tag and should be filtered out in the REQ.
    // It would only appear if the filter had not been updated correctly.
    let event = EventBuilder::text_note(":)")
        .tag(Tag::hashtag("TEST"))
        .finalize(&keys)
        .unwrap();
    client.send_event(&event).await.unwrap();

    // Empty filter to get all events. It should be updated to have `UPDATE_TAG`
    let events = client.fetch_events(Filter::new()).await.unwrap();

    assert!(!events.is_empty(), "Should not be empty");
    assert!(
        events
            .iter()
            .all(|e| { e.tags.hashtags().all(|hashtag| hashtag == UPDATE_TAG) }),
        "All tags should have the updated filter tag"
    );
}
