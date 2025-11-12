# Gossip in-memory storage

Reference `NostrGossip` implementation that stores relay metadata in an LRU cache. Ideal for bots or clients that want a drop-in gossip engine without running a database.

```rust,no_run
use std::num::NonZeroUsize;

use nostr::prelude::*;
use nostr_gossip::{BestRelaySelection, NostrGossip};
use nostr_gossip_memory::NostrGossipMemory;

# #[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let gossip = NostrGossipMemory::bounded(NonZeroUsize::new(2048).unwrap());
    let relay = RelayUrl::parse("wss://relay.primal.net")?;

    // Every event coming from your relay pool should be forwarded here
    let event = EventBuilder::text_note("demo note").sign_with_keys(&Keys::generate())?;
    gossip.process(&event, Some(&relay)).await?;

    // Later on, ask for the best relays for a profile
    let best = gossip
        .get_best_relays(
            &event.pubkey,
            BestRelaySelection::PrivateMessage { limit: 2 },
        )
        .await?;
    println!("DM relays -> {:?}", best);

    Ok(())
}
```

Use `NostrGossipMemory::unbounded()` for testing or small bots, and `bounded(limit)` to cap memory usage in long-running clients.

## Changelog

All notable changes to this library are documented in the [CHANGELOG.md](CHANGELOG.md).

## State

**This library is in an ALPHA state**, things that are implemented generally work but the API will change in breaking ways.

## Donations

`rust-nostr` is free and open-source. This means we do not earn any revenue by selling it. Instead, we rely on your financial support. If you actively use any of the `rust-nostr` libs/software/services, then please [donate](https://rust-nostr.org/donate).

## License

This project is distributed under the MIT software license - see the [LICENSE](../../LICENSE) file for details
