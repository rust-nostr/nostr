# Nostr SDK

[![crates.io](https://img.shields.io/crates/v/nostr-sdk.svg)](https://crates.io/crates/nostr-sdk)
[![Documentation](https://docs.rs/nostr-sdk/badge.svg)](https://docs.rs/nostr-sdk)
[![MIT](https://img.shields.io/crates/l/nostr-sdk.svg)](../../LICENSE)

## Description

A high-level, [Nostr](https://github.com/nostr-protocol/nostr) client library written in Rust.

If you're writing a typical Nostr client or bot, this is likely the crate you need.

However, the crate is designed in a modular way and depends on several
other lower-level crates. If you're attempting something more custom, you might be interested in these:

- [`nostr`](https://crates.io/crates/nostr): Rust implementation of Nostr protocol.

## Getting started

```toml
[dependencies]
anyhow = "1"
nostr-sdk = "0.7"
tokio = { version = "1", features = ["full"] }
url = "2"
```

```rust,no_run
use nostr_sdk::nostr::{Keys, Metadata};
use nostr_sdk::Client;
use url::Url;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Generate new keys
    let my_keys: Keys = Client::generate_keys();
    //
    // or use your already existing
    //
    // From Bech32
    // use nostr::key::FromBech32;
    // let my_keys = Keys::from_bech32("nsec1...")?;
    //
    // From hex string
    // use std::str::FromStr;
    // let my_keys = Keys::from_str("hex-secret-key")?;

    // Create new client
    let mut client = Client::new(&my_keys);

    // Add relays
    client.add_relay("wss://relay.damus.io", None)?;
    client.add_relay("wss://nostr.openchain.fr", None)?;

    // Connect to relays and keep connection alive
    client.connect().await?;

    let metadata = Metadata::new()
        .name("username")
        .display_name("My Username")
        .about("Description")
        .picture(Url::parse("https://example.com/avatar.png")?)
        .nip05("username@example.com");

    // Update profile metadata
    client.update_profile(metadata).await?;

    // Publish a text note
    client.publish_text_note("My first text note from Nostr SDK!", &[]).await?;

    // Publish a POW text note
    client.publish_pow_text_note("My first POW text note from Nostr SDK!", &[], 20).await?;

    // Handle notifications
    client
        .handle_notifications(|notification| {
            println!("{:?}", notification);
            Ok(())
        })
        .await
}
```

More examples can be found in the [examples](https://github.com/yukibtc/nostr-rs-sdk/tree/master/crates/nostr-sdk/examples) directory.

## Crate Feature Flags

The following crate feature flags are available:

| Feature             | Default | Description                                                                                                                |
| ------------------- | :-----: | -------------------------------------------------------------------------------------------------------------------------- |
| `blocking`          |   No    | Needed if you want to use this library in not async/await context                                                          |
| `all-nips`          |   No    | Enable all NIPs                                                                                                            |
| `nip04`             |   No    | Enable NIP-04: Encrypted Direct Message                                                                                    |
| `nip06`             |   No    | Enable NIP-06: Basic key derivation from mnemonic seed phrase                                                              |

## State

**This library is in an ALPHA state**, things that are implemented generally work but the API will change in breaking ways.

## License

This project is distributed under the MIT software license - see the [LICENSE](../../LICENSE) file for details