# Nostr

[![crates.io](https://img.shields.io/crates/v/nostr.svg)](https://crates.io/crates/nostr)
[![crates.io - Downloads](https://img.shields.io/crates/d/nostr)](https://crates.io/crates/nostr)
[![Documentation](https://docs.rs/nostr/badge.svg)](https://docs.rs/nostr)
[![CI](https://github.com/rust-nostr/nostr/actions/workflows/ci.yml/badge.svg)](https://github.com/rust-nostr/nostr/actions/workflows/ci.yml)
[![MIT](https://img.shields.io/crates/l/nostr.svg)](../../LICENSE)

## Description

Rust implementation of Nostr protocol.

You may be interested in:
* [`nostr-sdk`](https://crates.io/crates/nostr-sdk) if you want to write a typical Nostr client or bot
* [`nostr-relay-pool`](https://crates.io/crates/nostr-relay-pool): Nostr Relay Pool
* [`nostr-connect`](https://crates.io/crates/nostr-connect): Nostr Connect (NIP46)
* [`nwc`](https://crates.io/crates/nwc): Nostr Wallet Connect (NWC) client

## Getting started

```rust,no_run
use nostr::prelude::*;

fn main() -> Result<()> {
    // Generate new random keys
    let keys = Keys::generate();

    // Or use your already existing (from hex or bech32)
    let keys = Keys::parse("hex-or-bech32-secret-key")?;

    // Convert public key to bech32
    println!("Public key: {}", keys.public_key().to_bech32()?);

    let metadata = Metadata::new()
        .name("username")
        .display_name("My Username")
        .about("Description")
        .picture(Url::parse("https://example.com/avatar.png")?)
        .banner(Url::parse("https://example.com/banner.png")?)
        .nip05("username@example.com")
        .lud16("pay@yukikishimoto.com")
        .custom_field("custom_field", "my value");

    let event: Event = EventBuilder::metadata(&metadata).sign_with_keys(&keys)?;

    // New text note
    let event: Event = EventBuilder::text_note("Hello from rust-nostr").sign_with_keys(&keys)?;

    // New POW text note
    let event: Event = EventBuilder::text_note("POW text note from rust-nostr").pow(20).sign_with_keys(&keys)?;

    // Convert client message to JSON
    let json = ClientMessage::event(event).as_json();
    println!("{json}");

    Ok(())
}
```

More examples can be found in the [examples/](https://github.com/rust-nostr/nostr/tree/master/crates/nostr/examples) directory.

## WASM

This crate supports the `wasm32` targets.

On macOS you need to install `llvm`:

```shell
brew install llvm
LLVM_PATH=$(brew --prefix llvm)
AR="${LLVM_PATH}/bin/llvm-ar" CC="${LLVM_PATH}/bin/clang" cargo build --target wasm32-unknown-unknown
```

NOTE: Currently `nip03` feature not support WASM.

## Embedded

This crate support [`no_std`](https://docs.rust-embedded.org/book/intro/no-std.html) environments.

Check the example in the [embedded/](https://github.com/rust-nostr/nostr/tree/master/crates/nostr/examples/embedded) directory.

## Crate Feature Flags

The following crate feature flags are available:

| Feature            | Default | Description                                                   |
|--------------------|:-------:|---------------------------------------------------------------|
| `std`              |   Yes   | Enable `std` library                                          |
| `alloc`            |   No    | Needed to use this library in `no_std` context                |
| `pow-multi-thread` |   No    | Enable event POW mining using multi-threads                   |
| `all-nips`         |   No    | Enable all NIPs                                               |
| `nip03`            |   No    | Enable NIP-03: OpenTimestamps Attestations for Events         |
| `nip04`            |   No    | Enable NIP-04: Encrypted Direct Message                       |
| `nip06`            |   No    | Enable NIP-06: Basic key derivation from mnemonic seed phrase |
| `nip44`            |   No    | Enable NIP-44: Encrypted Payloads (Versioned)                 |
| `nip46`            |   No    | Enable NIP-46: Nostr Connect                                  |
| `nip47`            |   No    | Enable NIP-47: Nostr Wallet Connect                           |
| `nip49`            |   No    | Enable NIP-49: Private Key Encryption                         |
| `nip57`            |   No    | Enable NIP-57: Zaps                                           |
| `nip59`            |   No    | Enable NIP-59: Gift Wrap                                      |
| `nip60`            |   No    | Enable NIP-60: Cashu Wallets                                  |

## Changelog

All notable changes to this library are documented in the [CHANGELOG.md](CHANGELOG.md).

## State

**This library is in an ALPHA state**, things that are implemented generally work but the API will change in breaking ways.

## Donations

`rust-nostr` is free and open-source. This means we do not earn any revenue by selling it. Instead, we rely on your financial support. If you actively use any of the `rust-nostr` libs/software/services, then please [donate](https://rust-nostr.org/donate).

## License

This project is distributed under the MIT software license - see the [LICENSE](../../LICENSE) file for details
