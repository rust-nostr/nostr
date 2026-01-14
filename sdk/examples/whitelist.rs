// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::collections::HashSet;
use std::time::Duration;

use nostr_sdk::prelude::*;

#[derive(Debug, Default)]
struct WoT {
    allowed_public_keys: HashSet<PublicKey>,
}

impl AdmitPolicy for WoT {
    fn admit_event<'a>(
        &'a self,
        _relay_url: &'a RelayUrl,
        _subscription_id: &'a SubscriptionId,
        event: &'a Event,
    ) -> BoxedFuture<'a, Result<AdmitStatus, PolicyError>> {
        Box::pin(async move {
            if self.allowed_public_keys.contains(&event.pubkey) {
                return Ok(AdmitStatus::success());
            }

            Ok(AdmitStatus::rejected("Not in whitelist"))
        })
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let mut wot = WoT::default();

    // Allow public key
    let allowed_public_key =
        PublicKey::from_bech32("npub1drvpzev3syqt0kjrls50050uzf25gehpz9vgdw08hvex7e0vgfeq0eseet")?;
    wot.allowed_public_keys.insert(allowed_public_key);

    // Init client
    let client = Client::builder().admit_policy(wot).build();
    client.add_relay("wss://relay.damus.io").await?;
    client.connect().await;

    let not_in_whitelist_public_key =
        PublicKey::from_bech32("npub1xtscya34g58tk0z605fvr788k263gsu6cy9x0mhnm87echrgufzsevkk5s")?;

    // Get events from all connected relays
    let filter = Filter::new()
        .authors([allowed_public_key, not_in_whitelist_public_key])
        .kind(Kind::Metadata);
    let events = client.fetch_events(filter, Duration::from_secs(10)).await?;
    println!("Received {} events.", events.len());

    assert_eq!(events.len(), 1);
    assert_eq!(events.first().unwrap().pubkey, allowed_public_key);

    for event in events.into_iter() {
        println!("{}", event.as_json());
    }

    Ok(())
}
