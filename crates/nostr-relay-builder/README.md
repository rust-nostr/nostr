# Nostr Relay Builder

This library helps you stand up fully configurable relays without re-implementing policies, storage, or protocol minutiae. 

The crate exposes two main entry points:

- `LocalRelay` – run a fully fledged relay inside your process.
- `MockRelay` – run an ephemeral relay for unit/integration tests.

## Quick start

```rust,no_run
use nostr_lmdb::NostrLmdb;
use nostr_relay_builder::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Open a database (all databases that implements `NostrDatabase` trait can be used).
    let database = NostrLmdb::open("nostr-relay").await?;

    // Create the relay.
    let relay = LocalRelay::builder()
        .port(7777)
        .database(database)
        .rate_limit(RateLimit {
            max_reqs: 128,
            notes_per_minute: 30,
        })
        .build()?;

    // Start the relay.
    relay.run().await?;
    
    println!("Relay listening on {}", relay.url().await);

    // Keep the process running.
    tokio::signal::ctrl_c().await?;
    
    Ok(())
}
```

More examples can be found in the [examples directory](./examples).

## Supported NIPs

| Supported | NIP                                                                                                  |
|:---------:|------------------------------------------------------------------------------------------------------|
|     ✅     | [01 - Basic protocol flow description](https://github.com/nostr-protocol/nips/blob/master/01.md)     |
|     ✅     | [09 - Event Deletion](https://github.com/nostr-protocol/nips/blob/master/09.md)                      |
|     ❌     | [11 - Relay Information Document](https://github.com/nostr-protocol/nips/blob/master/11.md)          |
|     ✅     | [17 - Private Direct Messages](https://github.com/nostr-protocol/nips/blob/master/17.md)             |
|    🔧*    | [40 - Expiration Timestamp](https://github.com/nostr-protocol/nips/blob/master/40.md)                |
|     ✅     | [42 - Authentication of clients to relays](https://github.com/nostr-protocol/nips/blob/master/42.md) |
|    🔧     | [50 - Search Capability](https://github.com/nostr-protocol/nips/blob/master/50.md)                   |
|    🔧     | [62 - Request to Vanish](https://github.com/nostr-protocol/nips/blob/master/62.md)                   |
|     ✅     | [70 - Protected Events](https://github.com/nostr-protocol/nips/blob/master/70.md)                    |
|     ✅     | [77 - Negentropy Syncing](https://github.com/nostr-protocol/nips/blob/master/77.md)                  |

**Legend:**
- ✅ Fully supported
- 🔧 Depends on the database implementation
- ❌ Not supported

*: The relay does not accept or send expired events. The database have to delete them.

## Changelog

All notable changes to this library are documented in the [CHANGELOG.md](CHANGELOG.md).

## State

**This library is in an ALPHA state**, things that are implemented generally work but the API will change in breaking ways.

## Donations

`rust-nostr` is free and open-source. This means we do not earn any revenue by selling it. Instead, we rely on your financial support. If you actively use any of the `rust-nostr` libs/software/services, then please [donate](https://rust-nostr.org/donate).

## License

This project is distributed under the MIT software license - see the [LICENSE](../../LICENSE) file for details
