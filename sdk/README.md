# Nostr SDK

[![crates.io](https://img.shields.io/crates/v/nostr-sdk.svg)](https://crates.io/crates/nostr-sdk)
[![crates.io - Downloads](https://img.shields.io/crates/d/nostr-sdk)](https://crates.io/crates/nostr-sdk)
[![Documentation](https://docs.rs/nostr-sdk/badge.svg)](https://docs.rs/nostr-sdk)
[![CI](https://github.com/rust-nostr/nostr/actions/workflows/ci.yml/badge.svg)](https://github.com/rust-nostr/nostr/actions/workflows/ci.yml)
[![MIT](https://img.shields.io/crates/l/nostr-sdk.svg)](../LICENSE)

## Description

A full-featured SDK for building high-performance and reliable nostr applications.

## Getting started

```rust,no_run
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use nostr_sdk::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Generate new random keys
    let keys = Keys::generate();

    // Or use your already existing (from hex or bech32)
    let keys = Keys::parse("hex-or-bech32-secret-key")?;

    // Show bech32 public key
    let bech32_pubkey: String = keys.public_key().to_bech32()?;
    println!("Bech32 PubKey: {}", bech32_pubkey);

    // Configure client to use proxy for `.onion` relays
    let addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 9050));
    let connection: Connection = Connection::new()
        .proxy(addr) // Use `.embedded_tor()` instead to enable the embedded tor client (require `tor` feature)
        .target(ConnectionTarget::Onion);
    let client = Client::builder().signer(keys).connection(connection).build();

    // Add relays
    client.add_relay("wss://relay.damus.io").await?;
    client.add_relay("ws://jgqaglhautb4k6e6i2g34jakxiemqp6z4wynlirltuukgkft2xuglmqd.onion").await?;
    
    // Add read relay
    client.add_relay("wss://relay.nostr.info").capabilities(RelayCapabilities::READ).await?;

    // Connect to relays
    client.connect().await;

    // Publish a text note
    let builder = EventBuilder::text_note("My first text note from rust-nostr!");
    client.send_event_builder(builder).await?;

    // Create a POW text note
    let builder = EventBuilder::text_note("POW text note from nostr-sdk").pow(20);
    client.send_event_builder(builder).await?; // Send to all relays
    // client.send_event_builder_to(["wss://relay.damus.io"], builder).await?; // Send to specific relay

    Ok(())
}
```

More examples can be found in the [examples directory](./examples).

## Universal Time Provider

On `std` builds, `std::time` is used as the default time provider.

When compiling for `no_std` or `wasm*-unknown-unknown` targets, you may encounter the following linker error:
```text
error: undefined reference to '__universal_time_provider'
```

This error indicates that no time provider has been configured for the current target. 
In such environments, a time source is not available by default and must be supplied manually.

To resolve this, add and initialize a provider using the [universal-time](https://crates.io/crates/universal-time) library.

## WASM

This crate supports the `wasm32` targets.

An example can be found at [`nostr-sdk-wasm-example`](https://github.com/rust-nostr/nostr-sdk-wasm-example) repo.

On macOS, you need to install `llvm`:

```shell
brew install llvm
LLVM_PATH=$(brew --prefix llvm)
AR="${LLVM_PATH}/bin/llvm-ar" CC="${LLVM_PATH}/bin/clang" cargo build --target wasm32-unknown-unknown
```

NOTE: Currently `nip03` feature not support WASM.

## Changelog

All notable changes to this library are documented in the [CHANGELOG.md](CHANGELOG.md).

## State

**This library is in an ALPHA state**, things that are implemented generally work, but the API will change in breaking ways.

## Donations

`rust-nostr` is free and open-source. This means we do not earn any revenue by selling it. Instead, we rely on your financial support. If you actively use any of the `rust-nostr` libs/software/services, then please [donate](https://rust-nostr.org/donate).

## License

This project is distributed under the MIT software license – see the [LICENSE](../LICENSE) file for details
