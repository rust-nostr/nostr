# Nostr Relay Builder

`nostr-relay-builder` helps you stand up fully configurable relays (local or hidden-service) without re-implementing policies, storage, or protocol minutiae. The crate exposes two main entry points:

- `LocalRelay` – run a fully fledged relay inside your process.
- `MockRelay` – deterministic relay for unit/integration tests.

## Quick start

```rust,no_run
use std::net::Ipv4Addr;

use nostr::prelude::*;
use nostr_database::MemoryDatabase;
use nostr_relay_builder::{RelayBuilder, LocalRelay, RateLimit};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let relay = LocalRelay::new(
        RelayBuilder::default()
            .addr(Ipv4Addr::LOCALHOST.into())
            .port(7777)
            .database(MemoryDatabase::default())
            .rate_limit(RateLimit {
                max_reqs: 128,
                notes_per_minute: 30,
            }),
    );

    relay.run().await?;
    println!("relay listening on {}", relay.url().await);

    Ok(())
}
```

See the `local` and `mock` modules plus `examples/` for advanced policies such as:

- Enforcing NIP-42 auth via `RelayBuilder::nip42`.
- Only accepting writes from a given pubkey (`RelayBuilderMode::PublicKey`).
- Plugging in your own `NostrDatabase` backend and rate limits.
- Injecting events from tests via `MockRelay::notify_event`.

## Changelog

All notable changes to this library are documented in the [CHANGELOG.md](CHANGELOG.md).

## State

**This library is in an ALPHA state**, things that are implemented generally work but the API will change in breaking ways.

## Donations

`rust-nostr` is free and open-source. This means we do not earn any revenue by selling it. Instead, we rely on your financial support. If you actively use any of the `rust-nostr` libs/software/services, then please [donate](https://rust-nostr.org/donate).

## License

This project is distributed under the MIT software license - see the [LICENSE](../../LICENSE) file for details
