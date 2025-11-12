# Nostr Relay Pool

Nostr Relay Pool is the low-level building block used by `nostr-sdk` to manage many relay connections in parallel. Use it when you need fine-grained control over relay policies, admission rules, or when embedding the gossip stack in your own executor.

## Usage

```rust,no_run
use nostr::prelude::*;
use nostr_relay_pool::{RelayOptions, RelayPool, RelayPoolNotification};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = RelayPool::builder().build();

    // Add relays with custom options (timeouts, flags, etc.)
    pool.add_relay("wss://relay.damus.io", RelayOptions::default()).await?;
    pool.add_relay("wss://relay.primal.net", RelayOptions::default()).await?;

    // Fire up the background tasks and wait until we are connected
    pool.connect().await;
    pool.wait_for_connection(std::time::Duration::from_secs(5)).await;

    // Listen for broadcast notifications straight from the relays
    let mut notifications = pool.notifications();
    while let Ok(notification) = notifications.recv().await {
        if let RelayPoolNotification::Event { event, .. } = notification {
            println!("Got event {} -> {}", event.author(), event.content());
        }
    }

    Ok(())
}
```

See `crates/nostr-relay-pool/examples/` for more involved setups that mix monitors, sync policies, and custom transports.

## Crate Feature Flags

The following crate feature flags are available:

| Feature | Default | Description                               |
|---------|:-------:|-------------------------------------------|
| `tor`   |   No    | Enable support for embedded tor client    |

Enable the feature with `cargo add nostr-relay-pool --features tor` (native targets only).

## Changelog

All notable changes to this library are documented in the [CHANGELOG.md](CHANGELOG.md).

## State

**This library is in an ALPHA state**, things that are implemented generally work but the API will change in breaking ways.

## Donations

`rust-nostr` is free and open-source. This means we do not earn any revenue by selling it. Instead, we rely on your financial support. If you actively use any of the `rust-nostr` libs/software/services, then please [donate](https://rust-nostr.org/donate).

## License

This project is distributed under the MIT software license - see the [LICENSE](../../LICENSE) file for details
