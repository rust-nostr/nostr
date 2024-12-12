// Copyright (c) 2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::collections::HashSet;
use std::net::SocketAddr;
use std::time::Duration;

use nostr_relay_builder::prelude::*;

/// Accept only certain event kinds
#[derive(Debug)]
struct AcceptKinds {
    pub kinds: HashSet<Kind>,
}

#[async_trait]
impl WritePolicy for AcceptKinds {
    async fn admit_event(&self, event: &Event, _addr: &SocketAddr) -> PolicyResult {
        if self.kinds.contains(&event.kind) {
            PolicyResult::Accept
        } else {
            PolicyResult::Reject("kind not accepted".to_string())
        }
    }
}

/// Reject requests if there are more than [limit] authors in the filter
#[derive(Debug)]
struct RejectAuthorLimit {
    pub limit: usize,
}

#[async_trait]
impl QueryPolicy for RejectAuthorLimit {
    async fn admit_query(&self, query: &[Filter], _addr: &SocketAddr) -> PolicyResult {
        if query
            .iter()
            .any(|f| f.authors.as_ref().map(|a| a.len()).unwrap_or(0) > self.limit)
        {
            PolicyResult::Reject("query too expensive".to_string())
        } else {
            PolicyResult::Accept
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let accept_profile_data = AcceptKinds {
        kinds: HashSet::from([Kind::Metadata, Kind::RelayList, Kind::ContactList]),
    };

    let low_author_limit = RejectAuthorLimit { limit: 2 };

    let builder = RelayBuilder::default()
        .write_policy(accept_profile_data)
        .query_policy(low_author_limit);

    let relay = LocalRelay::run(builder).await?;

    println!("Url: {}", relay.url());

    // Keep up the program
    loop {
        tokio::time::sleep(Duration::from_secs(60)).await;
    }
}
