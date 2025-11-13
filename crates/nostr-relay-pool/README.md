# Nostr Relay Pool

This library is the low-level building block used by [`nostr-sdk`](../nostr-sdk) to manage relays.

If you’re just trying to write a nostr client or bot, you’re probably looking for [`nostr-sdk`](../nostr-sdk) instead.

## Usage

```rust,no_run
use std::time::Duration;

use nostr_relay_pool::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = RelayPool::new();

    // Add relays with custom options (timeouts, flags, etc.)
    pool.add_relay("wss://relay.damus.io", RelayOptions::default()).await?;
    pool.add_relay("wss://relay.primal.net", RelayOptions::default()).await?;

    // Fire up the background tasks and, optionally, wait until we are connected
    pool.connect().await;
    pool.wait_for_connection(Duration::from_secs(5)).await;

    // Listen for notifications
    let mut notifications = pool.notifications();
    while let Ok(notification) = notifications.recv().await {
        println!("Got notification: {:?}", notification);
    }

    Ok(())
}
```

More examples can be found in the [examples directory](./examples).

## Crate Feature Flags

The following crate feature flags are available:

| Feature | Default | Description                               |
|---------|:-------:|-------------------------------------------|
| `tor`   |   No    | Enable support for embedded tor client    |

## Changelog

All notable changes to this library are documented in the [CHANGELOG.md](CHANGELOG.md).

## State

**This library is in an ALPHA state**, things that are implemented generally work but the API will change in breaking ways.

## Donations

`rust-nostr` is free and open-source. This means we do not earn any revenue by selling it. Instead, we rely on your financial support. If you actively use any of the `rust-nostr` libs/software/services, then please [donate](https://rust-nostr.org/donate).

## License

This project is distributed under the MIT software license - see the [LICENSE](../../LICENSE) file for details
