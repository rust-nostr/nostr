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

- [`nostr`](https://crates.io/crates/nostr): Rust implementation of Nostr protocol
- [`nostr-sdk-net`](https://crates.io/crates/nostr-sdk-net): Nostr SDK Network library

## Getting started

```toml
[dependencies]
nostr-sdk = "0.24"
tokio = { version = "1", features = ["full"] }
```

```rust,no_run
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use nostr_sdk::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Generate new keys
    let my_keys: Keys = Keys::generate();
    //
    // or use your already existing
    //
    // From HEX or Bech32
    // let my_keys = Keys::from_sk_str("hex-or-bech32-secret-key")?;

    // Show bech32 public key
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
        .lud16("yuki@getalby.com")
        .custom_field("custom_field", Value::String("my value".into()));

    // Update metadata
    client.set_metadata(metadata).await?;

    // Publish a text note
    client.publish_text_note("My first text note from Nostr SDK!", &[]).await?;

    // Create a POW text note
    let event: Event = EventBuilder::new_text_note("POW text note from nostr-sdk", &[]).to_pow_event(&my_keys, 20)?;
    client.send_event(event).await?;

    // Send custom event
    let event_id = EventId::from_bech32("note1z3lwphdc7gdf6n0y4vaaa0x7ck778kg638lk0nqv2yd343qda78sf69t6r")?;
    let public_key = XOnlyPublicKey::from_bech32("npub14rnkcwkw0q5lnmjye7ffxvy7yxscyjl3u4mrr5qxsks76zctmz3qvuftjz")?;
    let event: Event = EventBuilder::new_reaction(event_id, public_key, "🧡").to_event(&my_keys)?;

    // Send custom event to all relays
    // client.send_event(event).await?;

    // Send custom event to a specific previously added relay
    client.send_event_to("wss://relay.damus.io", event).await?;

    Ok(())
}
```

More examples can be found in the [examples/](https://github.com/rust-nostr/nostr/tree/master/crates/nostr-sdk/examples) directory.

## WASM

This crate supports the `wasm32` targets.

An example can be found at [`nostr-sdk-wasm-example`](https://github.com/NostrDevKit/nostr-sdk-wasm-example) repo.

On macOS you need to install `llvm`:

```shell
brew install llvm
LLVM_PATH=$(brew --prefix llvm)
AR="${LLVM_PATH}/bin/llvm-ar" CC="${LLVM_PATH}/bin/clang" cargo build --target wasm32-unknown-unknown
```

NOTE: Currently `nip03` feature not support WASM.

## Crate Feature Flags

The following crate feature flags are available:

| Feature             | Default | Description                                                                              |
| ------------------- | :-----: | ---------------------------------------------------------------------------------------- |
| `blocking`          |   No    | Needed to use `NIP-05` and `NIP-11` features in not async/await context                  |
| `all-nips`          |   Yes   | Enable all NIPs                                                                          |
| `nip03`             |   No    | Enable NIP-03: OpenTimestamps Attestations for Events                                    |
| `nip04`             |   Yes   | Enable NIP-04: Encrypted Direct Message                                                  |
| `nip05`             |   Yes   | Enable NIP-05: Mapping Nostr keys to DNS-based internet identifiers                      |
| `nip06`             |   Yes   | Enable NIP-06: Basic key derivation from mnemonic seed phrase                            |
| `nip11`             |   Yes   | Enable NIP-11: Relay Information Document                                                |
| `nip44`             |   No    | Enable NIP-44: Encrypted Payloads (Versioned) - EXPERIMENTAL                             |
| `nip46`             |   Yes   | Enable NIP-46: Nostr Connect                                                             |
| `nip47`             |   Yes   | Enable NIP-47: Nostr Wallet Connect                                                      |

## Supported NIPs

Look at <https://github.com/rust-nostr/nostr/tree/master/crates/nostr#supported-nips>

## State

**This library is in an ALPHA state**, things that are implemented generally work but the API will change in breaking ways.

## License

This project is distributed under the MIT software license - see the [LICENSE](../../LICENSE) file for details

## Donations

⚡ Tips: <https://getalby.com/p/yuki>

⚡ Lightning Address: yuki@getalby.com