# Nostr gossip traits

Core traits and utility types for tracking relay lists (NIP-65), inbox relays (NIP-17), and best-relay selection heuristics. Implement the `NostrGossip` trait to plug custom storage engines into `nostr-sdk` or run the provided in-memory store from `nostr-gossip-memory`.

## Usage

```rust,no_run
use std::num::NonZeroUsize;

use nostr::prelude::*;
use nostr_gossip::{BestRelaySelection, GossipListKind, NostrGossip};
use nostr_gossip_memory::NostrGossipMemory;

# #[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let gossip = NostrGossipMemory::bounded(NonZeroUsize::new(1024).unwrap());
    let relay = RelayUrl::parse("wss://relay.damus.io")?;

    // Feed the store with events as they arrive from your pool/client
    let event = EventBuilder::text_note("hello").sign_with_keys(&Keys::generate())?;
    gossip.process(&event, Some(&relay)).await?;

    // Check if we need to refresh metadata for a pubkey
    if matches!(
        gossip
            .status(&event.pubkey, GossipListKind::Nip65)
            .await?,
        nostr_gossip::GossipPublicKeyStatus::Outdated { .. }
    ) {
        // trigger a sync
    }

    // Ask for the best relays to read from
    let relays = gossip
        .get_best_relays(
            &event.pubkey,
            BestRelaySelection::Read { limit: 2 },
        )
        .await?;
    println!("{} prefers {:?}", event.pubkey, relays);

    Ok(())
}
```

The crate also exposes flags for scoring relays (`GossipFlags`) and helper enums for targeting read/write/private message selections.

## Changelog

All notable changes to this library are documented in the [CHANGELOG.md](CHANGELOG.md).

## State

**This library is in an ALPHA state**, things that are implemented generally work but the API will change in breaking ways.

## Donations

`rust-nostr` is free and open-source. This means we do not earn any revenue by selling it. Instead, we rely on your financial support. If you actively use any of the `rust-nostr` libs/software/services, then please [donate](https://rust-nostr.org/donate).

## License

This project is distributed under the MIT software license - see the [LICENSE](../../LICENSE) file for details
