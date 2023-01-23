# Nostr SDK

[![crates.io](https://img.shields.io/crates/v/nostr-sdk.svg)](https://crates.io/crates/nostr-sdk)
[![crates.io - Downloads](https://img.shields.io/crates/d/nostr-sdk)](https://crates.io/crates/nostr-sdk)
[![Documentation](https://docs.rs/nostr-sdk/badge.svg)](https://docs.rs/nostr-sdk)
[![Rustc Version 1.64.0+](https://img.shields.io/badge/rustc-1.64.0%2B-lightgrey.svg)](https://blog.rust-lang.org/2022/09/22/Rust-1.64.0.html)
[![CI](https://github.com/rust-nostr/nostr/actions/workflows/ci.yml/badge.svg)](https://github.com/rust-nostr/nostr/actions/workflows/ci.yml)
[![MIT](https://img.shields.io/crates/l/nostr-sdk.svg)](../../LICENSE)
![Lines of code](https://img.shields.io/tokei/lines/github/rust-nostr/nostr)

## Description

A high-level, [Nostr](https://github.com/nostr-protocol/nostr) client library written in Rust.

If you're writing a typical Nostr client or bot, this is likely the crate you need.

However, the crate is designed in a modular way and depends on several
other lower-level crates. If you're attempting something more custom, you might be interested in these:

- [`nostr`](https://crates.io/crates/nostr): Rust implementation of Nostr protocol.

## Getting started

```toml
[dependencies]
nostr-sdk = "0.15"
tokio = { version = "1", features = ["full"] }
```

```rust,no_run
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use nostr_sdk::nostr::{Keys, Metadata, Url};
use nostr_sdk::{Client, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // Generate new keys
    let my_keys: Keys = Client::generate_keys();
    //
    // or use your already existing
    //
    // From HEX or Bech32
    // use nostr_sdk::nostr::key::FromSkStr;
    // let my_keys = Keys::from_sk_str("hex-or-bech32-secret-key")?;

    // Show bech32 public key
    use nostr_sdk::nostr::nips::nip19::ToBech32;
    let bech32_pubkey: String = my_keys.public_key().to_bech32()?;
    println!("Bech32 PubKey: {}", bech32_pubkey);

    // Create new client
    let client = Client::new(&my_keys);

    let proxy = Some(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 9050)));

    // Add relays
    client.add_relay("wss://relay.damus.io", None).await?;
    client.add_relay("wss://relay.nostr.info", proxy).await?;
    client.add_relay(
        "ws://jgqaglhautb4k6e6i2g34jakxiemqp6z4wynlirltuukgkft2xuglmqd.onion",
        proxy,
    ).await?;

    // Connect to relays
    client.connect().await;

    let metadata = Metadata::new()
        .name("username")
        .display_name("My Username")
        .about("Description")
        .picture(Url::parse("https://example.com/avatar.png")?)
        .banner(Url::parse("https://example.com/banner.png")?)
        .nip05("username@example.com")
        .lud16("yuki@stacker.news");

    // Update profile metadata
    client.update_profile(metadata).await?;

    // Publish a text note
    client.publish_text_note("My first text note from Nostr SDK!", &[]).await?;

    // Publish a POW text note
    client.publish_pow_text_note("My first POW text note from Nostr SDK!", &[], 20).await?;

    // Handle notifications
    loop {
        let mut notifications = client.notifications();
        while let Ok(notification) = notifications.recv().await {
            println!("{:?}", notification);
        }
    }
}
```

More examples can be found in the [examples](https://github.com/rust-nostr/nostr/tree/master/crates/nostr-sdk/examples) directory.

## Crate Feature Flags

The following crate feature flags are available:

| Feature             | Default | Description                                                                                                                |
| ------------------- | :-----: | -------------------------------------------------------------------------------------------------------------------------- |
| `blocking`          |   No    | Needed to use this library in not async/await context                                                                      |
| `all-nips`          |   Yes   | Enable all NIPs                                                                                                            |
| `nip04`             |   Yes   | Enable NIP-04: Encrypted Direct Message                                                                                    |
| `nip05`             |   Yes   | Enable NIP-05: Mapping Nostr keys to DNS-based internet identifiers                                                        |
| `nip06`             |   Yes   | Enable NIP-06: Basic key derivation from mnemonic seed phrase                                                              |
| `nip11`             |   Yes   | Enable NIP-11: Relay Information Document                                                                                  |
| `nip13`             |   Yes   | Enable NIP-13: Proof of Work                                                                                               |
| `nip19`             |   Yes   | Enable NIP-19: bech32-encoded entities                                                                                     |
| `nip26`             |   Yes   | Enable NIP-26: Delegated Event Signing                                                                                     |

## Supported NIPs

Look at https://github.com/rust-nostr/nostr/tree/master/crates/nostr#supported-nips

## State

**This library is in an ALPHA state**, things that are implemented generally work but the API will change in breaking ways.

## License

This project is distributed under the MIT software license - see the [LICENSE](../../LICENSE) file for details

## Donations

⚡ Tips: https://getalby.com/p/yuki

⚡ Lightning Address: yuki@getalby.com