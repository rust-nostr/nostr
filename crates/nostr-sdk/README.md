# Nostr SDK

A high-level, [Nostr](https://github.com/nostr-protocol/nostr) client library written in Rust.

If you're writing a typical Nostr client or bot, this is likely the crate you need.

However, the crate is designed in a modular way and depends on several
other lower-level crates. If you're attempting something more custom, you might be interested in these:

- [`nostr-sdk-base`](https://crates.io/crates/nostr-sdk-base): Rust implementation of Nostr protocol.

## Getting started

```toml
[dependencies]
anyhow = "1"
nostr-sdk = "0.1"
tokio = { version = "1", features = ["full"] }
```

```rust,no_run
use nostr_sdk::base::{Event, Keys};
use nostr_sdk::Client;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Init logger
    env_logger::init();

    // Generate new keys
    let my_keys: Keys = Client::generate_keys();
    //
    // or use your already existing
    //
    // use nostr_sdk::base::key::FromBech32;
    // let my_keys: Keys = Keys::from_bech32("nsec1...")?;

    // Create new client
    let mut client = Client::new(&my_keys);

    // Add relays
    client.add_relay("wss://relay.damus.io", None)?;
    client.add_relay("wss://nostr.openchain.fr", None)?;

    // Connect to relays and keep alive connection
    client.connect().await?;

    // Update profile metadata
    client.update_profile(
        Some("username"), 
        Some("Display Name"), 
        Some("About"), 
        Some("https://example.com/avatar.png")
    ).await?;

    // Publish a text note
    client.publish_text_note("My first text note from Nostr SDK!", &[]).await?;

    // Publish a POW text note
    client.publish_pow_text_note("My first text note from Nostr SDK!", &[], 16).await?;

    // Disconnect from all relays
    client.disconnect().await?;

    Ok(())
}
```

More examples can be found in the [examples](https://github.com/yukibtc/nostr-rs-sdk/tree/master/crates/nostr-sdk/examples) directory.

## Crate Feature Flags

The following crate feature flags are available:

| Feature             | Default | Description                                                                                                                |
| ------------------- | :-----: | -------------------------------------------------------------------------------------------------------------------------- |
| `blocking`          |   No    | Needed if you want to use this library in not async/await context                                                          |

## State

**This library is in an ALPHA state**, things that are implemented generally work but the API will change in breaking ways.

## License

This project is distributed under the MIT software license - see the [LICENSE](LICENSE) file for details