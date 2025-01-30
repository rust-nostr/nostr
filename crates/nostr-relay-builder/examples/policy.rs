// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
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

impl WritePolicy for AcceptKinds {
    fn admit_event<'a>(
        &'a self,
        event: &'a Event,
        _addr: &'a SocketAddr,
    ) -> BoxedFuture<'a, PolicyResult> {
        Box::pin(async move {
            if self.kinds.contains(&event.kind) {
                PolicyResult::Accept
            } else {
                PolicyResult::Reject("kind not accepted".to_string())
            }
        })
    }
}

/// Reject requests if there are more than [limit] authors in the filter
#[derive(Debug)]
struct RejectAuthorLimit {
    pub limit: usize,
}

impl QueryPolicy for RejectAuthorLimit {
    fn admit_query<'a>(
        &'a self,
        query: &'a Filter,
        _addr: &'a SocketAddr,
    ) -> BoxedFuture<'a, PolicyResult> {
        Box::pin(async move {
            if query.authors.as_ref().map(|a| a.len()).unwrap_or(0) > self.limit {
                PolicyResult::Reject("query too expensive".to_string())
            } else {
                PolicyResult::Accept
            }
        })
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
