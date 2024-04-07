# Nostr SDK

[![crates.io](https://img.shields.io/crates/v/nostr-sdk.svg)](https://crates.io/crates/nostr-sdk)
[![crates.io - Downloads](https://img.shields.io/crates/d/nostr-sdk)](https://crates.io/crates/nostr-sdk)
[![Documentation](https://docs.rs/nostr-sdk/badge.svg)](https://docs.rs/nostr-sdk)
[![Rustc Version 1.64.0+](https://img.shields.io/badge/rustc-1.64.0%2B-lightgrey.svg)](https://blog.rust-lang.org/2022/09/22/Rust-1.64.0.html)
[![CI](https://github.com/rust-nostr/nostr/actions/workflows/ci.yml/badge.svg)](https://github.com/rust-nostr/nostr/actions/workflows/ci.yml)
[![MIT](https://img.shields.io/crates/l/nostr-sdk.svg)](../../LICENSE)

## Description

A high-level, [Nostr](https://github.com/nostr-protocol/nostr) client library written in Rust.

If you're writing a typical Nostr client or bot, this is likely the crate you need.

However, the crate is designed in a modular way and depends on several
other lower-level crates. If you're attempting something more custom, you might be interested in these:

* [`nostr`](https://crates.io/crates/nostr): Rust implementation of Nostr protocol
* [`nostr-database`](https://crates.io/crates/nostr-database): Database for Nostr apps
* [`nostr-relay-pool`](https://crates.io/crates/nostr-relay-pool): Nostr Relay Pool
* [`nostr-signer`](https://crates.io/crates/nostr-signer): Signer for Nostr apps
* [`nostr-zapper`](https://crates.io/crates/nostr-zapper): Zapper abscraction for Nostr apps
* [`nwc`](https://crates.io/crates/nwc): `NWC` client and zapper backend for Nostr apps

## Getting started

```toml
[dependencies]
nostr-sdk = "0.29"
tokio = { version = "1", features = ["full"] }
```

```rust,no_run
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::str::FromStr;

use nostr_sdk::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Generate new random keys
    let my_keys = Keys::generate();

    // Or use your already existing (from hex or bech32)
    let my_keys = Keys::parse("hex-or-bech32-secret-key")?;

    // Show bech32 public key
    let bech32_pubkey: String = my_keys.public_key().to_bech32();
    println!("Bech32 PubKey: {}", bech32_pubkey);

    // Create new client
    let client = Client::new(&my_keys);

    let proxy = Some(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 9050)));

    // Add relays
    client.add_relay("wss://relay.damus.io").await?;
    client.add_relay_with_opts(
        "wss://relay.nostr.info", 
        RelayOptions::new().proxy(proxy).write(false)
    ).await?;
    client.add_relay_with_opts(
        "ws://jgqaglhautb4k6e6i2g34jakxiemqp6z4wynlirltuukgkft2xuglmqd.onion",
        RelayOptions::new().proxy(proxy),
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
        .custom_field("custom_field", "my value");

    // Update metadata
    client.set_metadata(&metadata).await?;

    // Publish a text note
    client.publish_text_note("My first text note from Nostr SDK!", []).await?;

    // Create a POW text note
    let event: Event = EventBuilder::text_note("POW text note from nostr-sdk", []).to_pow_event(&my_keys, 20)?;
    client.send_event(event).await?; // Send to all relays
    // client.send_event_to(["wss://relay.damus.io"], event).await?; // Send to specific relay

    // --------- Zap! -------------

    // Configure zapper
    let uri = NostrWalletConnectURI::from_str("nostr+walletconnect://...")?;
    let zapper = NWC::new(uri).await?; // Use `WebLNZapper::new().await` for WebLN
    client.set_zapper(zapper).await;

    // Send SAT without zap event
    let public_key = PublicKey::from_bech32(
        "npub1drvpzev3syqt0kjrls50050uzf25gehpz9vgdw08hvex7e0vgfeq0eseet",
    )?;
    client.zap(public_key, 1000, None).await?;

    // Zap profile
    let details = ZapDetails::new(ZapType::Public).message("Test");
    client.zap(public_key, 1000, Some(details)).await?;

    // Zap event
    let event = Nip19Event::from_bech32("nevent1qqsr0q447ylm3y3tvw07vt69w3kzk026vl6yn3dwm9fweay0dw0jttgpz3mhxue69uhhyetvv9ujumn0wd68ytnzvupzq6xcz9jerqgqkldy8lpg7lglcyj4g3nwzy2cs6u70wejdaj7csnjqvzqqqqqqygequ53")?;
    let details = ZapDetails::new(ZapType::Anonymous).message("Anonymous Zap!");
    client.zap(event, 1000, Some(details)).await?;

    Ok(())
}
```

More examples can be found in the [examples/](https://github.com/rust-nostr/nostr/tree/master/crates/nostr-sdk/examples) directory.

## WASM

This crate supports the `wasm32` targets.

An example can be found at [`nostr-sdk-wasm-example`](https://github.com/rust-nostr/nostr-sdk-wasm-example) repo.

On macOS you need to install `llvm`:

```shell
brew install llvm
LLVM_PATH=$(brew --prefix llvm)
AR="${LLVM_PATH}/bin/llvm-ar" CC="${LLVM_PATH}/bin/clang" cargo build --target wasm32-unknown-unknown
```

NOTE: Currently `nip03` feature not support WASM.

## Crate Feature Flags

The following crate feature flags are available:

| Feature             | Default | Description                                                                                 |
| ------------------- | :-----: | ------------------------------------------------------------------------------------------- |
| `sqlite`            |   No    | Enable SQLite Storage backend                                                               |
| `rocksdb`           |   No    | Enable RocksDB Storage backend                                                              |
| `indexeddb`         |   No    | Enable Web's IndexedDb Storage backend                                                      |
| `webln`             |   No    | Enable WebLN zapper                                                                         |
| `all-nips`          |   Yes   | Enable all NIPs                                                                             |
| `nip03`             |   No    | Enable NIP-03: OpenTimestamps Attestations for Events                                       |
| `nip04`             |   Yes   | Enable NIP-04: Encrypted Direct Message                                                     |
| `nip05`             |   Yes   | Enable NIP-05: Mapping Nostr keys to DNS-based internet identifiers                         |
| `nip06`             |   Yes   | Enable NIP-06: Basic key derivation from mnemonic seed phrase                               |
| `nip07`             |   Yes   | Enable NIP-07: `window.nostr` capability for web browsers (**available only for `wasm32`!**)|
| `nip11`             |   Yes   | Enable NIP-11: Relay Information Document                                                   |
| `nip44`             |   Yes   | Enable NIP-44: Encrypted Payloads (Versioned)                                               |
| `nip46`             |   Yes   | Enable NIP-46: Nostr Connect                                                                |
| `nip47`             |   Yes   | Enable NIP-47: Nostr Wallet Connect                                                         |
| `nip49`             |   Yes   | Enable NIP-49: Private Key Encryption                                                       |
| `nip57`             |   Yes   | Enable NIP-57: Zaps                                                                         |
| `nip59`             |   Yes   | Enable NIP-59: Gift Wrap                                                                    |

## Supported NIPs

Look at <https://github.com/rust-nostr/nostr/tree/master/crates/nostr#supported-nips>

## State

**This library is in an ALPHA state**, things that are implemented generally work but the API will change in breaking ways.

## Donations

`rust-nostr` is free and open-source. This means we do not earn any revenue by selling it. Instead, we rely on your financial support. If you actively use any of the `rust-nostr` libs/software/services, then please [donate](https://rust-nostr.org/donate).

## License

This project is distributed under the MIT software license - see the [LICENSE](../../LICENSE) file for details