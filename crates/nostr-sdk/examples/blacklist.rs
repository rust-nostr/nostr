// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::collections::HashSet;
use std::time::Duration;

use nostr_sdk::prelude::*;

#[derive(Debug, Default)]
struct Filtering {
    muted_public_keys: HashSet<PublicKey>,
}

impl AdmitPolicy for Filtering {
    fn admit_event<'a>(
        &'a self,
        _relay_url: &'a RelayUrl,
        _subscription_id: &'a SubscriptionId,
        event: &'a Event,
    ) -> BoxedFuture<'a, Result<AdmitStatus, PolicyError>> {
        Box::pin(async move {
            if self.muted_public_keys.contains(&event.pubkey) {
                return Ok(AdmitStatus::rejected("Muted"));
            }

            Ok(AdmitStatus::success())
        })
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let mut filtering = Filtering::default();

    // Mute public key
    let muted_public_key =
        PublicKey::from_bech32("npub1l2vyh47mk2p0qlsku7hg0vn29faehy9hy34ygaclpn66ukqp3afqutajft")?;
    filtering.muted_public_keys.insert(muted_public_key);

    // Init client
    let client = Client::builder().admit_policy(filtering).build();
    client.add_relay("wss://relay.damus.io").await?;
    client.connect().await;

    // Get events from all connected relays
    let public_key =
        PublicKey::from_bech32("npub1xtscya34g58tk0z605fvr788k263gsu6cy9x0mhnm87echrgufzsevkk5s")?;
    let filter = Filter::new()
        .authors([muted_public_key, public_key])
        .kind(Kind::Metadata);
    let events = client.fetch_events(filter, Duration::from_secs(10)).await?;
    println!("Received {} events.", events.len());

    assert_eq!(events.len(), 1);
    assert_eq!(events.first_owned().unwrap().pubkey, public_key);

    Ok(())
}
