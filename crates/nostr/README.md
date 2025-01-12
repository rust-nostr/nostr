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

| Feature    | Default | Description                                                                                  |
|------------|:-------:|----------------------------------------------------------------------------------------------|
| `std`      |   Yes   | Enable `std` library                                                                         |
| `alloc`    |   No    | Needed to use this library in `no_std` context                                               |
| `all-nips` |   No    | Enable all NIPs                                                                              |
| `nip03`    |   No    | Enable NIP-03: OpenTimestamps Attestations for Events                                        |
| `nip04`    |   No    | Enable NIP-04: Encrypted Direct Message                                                      |
| `nip05`    |   No    | Enable NIP-05: Mapping Nostr keys to DNS-based internet identifiers                          |
| `nip06`    |   No    | Enable NIP-06: Basic key derivation from mnemonic seed phrase                                |
| `nip07`    |   No    | Enable NIP-07: `window.nostr` capability for web browsers (**available only for `wasm32`!**) |
| `nip11`    |   No    | Enable NIP-11: Relay Information Document                                                    |
| `nip44`    |   No    | Enable NIP-44: Encrypted Payloads (Versioned)                                                |
| `nip46`    |   No    | Enable NIP-46: Nostr Connect                                                                 |
| `nip47`    |   No    | Enable NIP-47: Nostr Wallet Connect                                                          |
| `nip49`    |   No    | Enable NIP-49: Private Key Encryption                                                        |
| `nip57`    |   No    | Enable NIP-57: Zaps                                                                          |
| `nip59`    |   No    | Enable NIP-59: Gift Wrap                                                                     |

## Supported NIPs

| Supported | NIP                                                                                                             |
|:---------:|-----------------------------------------------------------------------------------------------------------------|
|     ✅     | [01 - Basic protocol flow description](https://github.com/nostr-protocol/nips/blob/master/01.md)                |
|     ✅     | [02 - Follow List](https://github.com/nostr-protocol/nips/blob/master/02.md)                                    |
|     ✅     | [03 - OpenTimestamps Attestations for Events](https://github.com/nostr-protocol/nips/blob/master/03.md)         |
|     ✅     | [04 - Encrypted Direct Message](https://github.com/nostr-protocol/nips/blob/master/04.md)                       |
|     ✅     | [05 - Mapping Nostr keys to DNS-based internet ids](https://github.com/nostr-protocol/nips/blob/master/05.md)   |
|     ✅     | [06 - Basic key derivation from mnemonic seed phrase](https://github.com/nostr-protocol/nips/blob/master/06.md) |
|     ✅     | [07 - `window.nostr` capability for web browsers](https://github.com/nostr-protocol/nips/blob/master/07.md)     |
|     ✅     | [09 - Event Deletion](https://github.com/nostr-protocol/nips/blob/master/09.md)                                 |
|     ✅     | [10 - Use of `e` and `p` tags in text events](https://github.com/nostr-protocol/nips/blob/master/10.md)         |
|     ✅     | [11 - Relay Information Document](https://github.com/nostr-protocol/nips/blob/master/11.md)                     |
|     ✅     | [13 - Proof of Work](https://github.com/nostr-protocol/nips/blob/master/13.md)                                  |
|     ✅     | [14 - Subject tag in text events](https://github.com/nostr-protocol/nips/blob/master/14.md)                     |
|     ✅     | [15 - Nostr Marketplace](https://github.com/nostr-protocol/nips/blob/master/15.md)                              |
|     ✅     | [17 - Private Direct Messages](https://github.com/nostr-protocol/nips/blob/master/17.md)                        |
|     ✅     | [18 - Reposts](https://github.com/nostr-protocol/nips/blob/master/18.md)                                        |
|     ✅     | [19 - bech32-encoded entities](https://github.com/nostr-protocol/nips/blob/master/19.md)                        |
|     ✅     | [21 - URI scheme](https://github.com/nostr-protocol/nips/blob/master/21.md)                                     |
|     ✅     | [22 - Comment](https://github.com/nostr-protocol/nips/blob/master/22.md)                                        |
|     ✅     | [23 - Long-form Content](https://github.com/nostr-protocol/nips/blob/master/23.md)                              |
|     ✅     | [24 - Extra metadata fields and tags](https://github.com/nostr-protocol/nips/blob/master/24.md)                 |
|     ✅     | [25 - Reactions](https://github.com/nostr-protocol/nips/blob/master/25.md)                                      |
|     ✅     | [26 - Delegated Event Signing](https://github.com/nostr-protocol/nips/blob/master/26.md)                        |
|     ❌     | [27 - Text Note References](https://github.com/nostr-protocol/nips/blob/master/27.md)                           |
|     ✅     | [28 - Public Chat](https://github.com/nostr-protocol/nips/blob/master/28.md)                                    |
|     ❌     | [29 - Relay-based Groups](https://github.com/nostr-protocol/nips/blob/master/29.md)                             |
|     ✅     | [30 - Custom Emoji](https://github.com/nostr-protocol/nips/blob/master/30.md)                                   |
|     ✅     | [31 - Dealing with Unknown Events](https://github.com/nostr-protocol/nips/blob/master/31.md)                    |
|     ✅     | [32 - Labeling](https://github.com/nostr-protocol/nips/blob/master/32.md)                                       |
|     ✅     | [34 - `git` stuff](https://github.com/nostr-protocol/nips/blob/master/34.md)                                    |
|     ✅     | [35 - Torrents](https://github.com/nostr-protocol/nips/blob/master/35.md)                                       |
|     ✅     | [36 - Sensitive Content](https://github.com/nostr-protocol/nips/blob/master/36.md)                              |
|     ❌     | [38 - User Statuses](https://github.com/nostr-protocol/nips/blob/master/38.md)                                  |
|     ✅     | [39 - External Identities in Profiles](https://github.com/nostr-protocol/nips/blob/master/39.md)                |
|     ✅     | [40 - Expiration Timestamp](https://github.com/nostr-protocol/nips/blob/master/40.md)                           |
|     ✅     | [42 - Authentication of clients to relays](https://github.com/nostr-protocol/nips/blob/master/42.md)            |
|     ✅     | [44 - Encrypted Payloads (Versioned)](https://github.com/nostr-protocol/nips/blob/master/44.md)                 |
|     ✅     | [45 - Event Counts](https://github.com/nostr-protocol/nips/blob/master/45.md)                                   |
|     ✅     | [46 - Nostr Connect](https://github.com/nostr-protocol/nips/blob/master/46.md)                                  |
|     ✅     | [47 - Wallet Connect](https://github.com/nostr-protocol/nips/blob/master/47.md)                                 |
|     ✅     | [48 - Proxy Tags](https://github.com/nostr-protocol/nips/blob/master/48.md)                                     |
|     ✅     | [49 - Private Key Encryption](https://github.com/nostr-protocol/nips/blob/master/49.md)                         |
|     ✅     | [50 - Search Capability](https://github.com/nostr-protocol/nips/blob/master/50.md)                              |
|     ✅     | [51 - Lists](https://github.com/nostr-protocol/nips/blob/master/51.md)                                          |
|     ❌     | [52 - Calendar Events](https://github.com/nostr-protocol/nips/blob/master/52.md)                                |
|     ✅     | [53 - Live Activities](https://github.com/nostr-protocol/nips/blob/master/53.md)                                |
|     ❌     | [54 - Wiki](https://github.com/nostr-protocol/nips/blob/master/54.md)                                           |
|     -     | [55 - Android Signer Application](https://github.com/nostr-protocol/nips/blob/master/55.md)                     |
|     ✅     | [56 - Reporting](https://github.com/nostr-protocol/nips/blob/master/56.md)                                      |
|     ✅     | [57 - Lightning Zaps](https://github.com/nostr-protocol/nips/blob/master/57.md)                                 |
|     ✅     | [58 - Badges](https://github.com/nostr-protocol/nips/blob/master/58.md)                                         |
|     ✅     | [59 - Gift Wrap](https://github.com/nostr-protocol/nips/blob/master/59.md)                                      |
|     ✅     | [65 - Relay List Metadata](https://github.com/nostr-protocol/nips/blob/master/65.md)                            |
|     ✅     | [70 - Protected Events](https://github.com/nostr-protocol/nips/blob/master/70.md)                               |
|     ❌     | [71 - Video Events](https://github.com/nostr-protocol/nips/blob/master/71.md)                                   |
|     ❌     | [72 - Moderated Communities](https://github.com/nostr-protocol/nips/blob/master/72.md)                          |
|     ✅     | [73 - External Content IDs](https://github.com/nostr-protocol/nips/blob/master/73.md)                           |
|     ❌     | [75 - Zap Goals](https://github.com/nostr-protocol/nips/blob/master/75.md)                                      |
|     ✅     | [78 - Arbitrary custom app data](https://github.com/nostr-protocol/nips/blob/master/78.md)                      |
|     ❌     | [89 - Recommended Application Handlers](https://github.com/nostr-protocol/nips/blob/master/89.md)               |
|     ✅     | [90 - Data Vending Machine](https://github.com/nostr-protocol/nips/blob/master/90.md)                           |
|     ❌     | [92 - Media Attachments](https://github.com/nostr-protocol/nips/blob/master/92.md)                              |
|     ✅     | [94 - File Metadata](https://github.com/nostr-protocol/nips/blob/master/94.md)                                  |
|     ✅     | [96 - HTTP File Storage Integration](https://github.com/nostr-protocol/nips/blob/master/96.md)                  |
|     ✅     | [98 - HTTP Auth](https://github.com/nostr-protocol/nips/blob/master/98.md)                                      |
|     ❌     | [99 - Classified Listings](https://github.com/nostr-protocol/nips/blob/master/99.md)                            |

## State

**This library is in an ALPHA state**, things that are implemented generally work but the API will change in breaking ways.

## Donations

`rust-nostr` is free and open-source. This means we do not earn any revenue by selling it. Instead, we rely on your financial support. If you actively use any of the `rust-nostr` libs/software/services, then please [donate](https://rust-nostr.org/donate).

## License

This project is distributed under the MIT software license - see the [LICENSE](../../LICENSE) file for details
