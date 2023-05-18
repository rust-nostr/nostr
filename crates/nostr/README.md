# Nostr

[![crates.io](https://img.shields.io/crates/v/nostr.svg)](https://crates.io/crates/nostr)
[![crates.io - Downloads](https://img.shields.io/crates/d/nostr)](https://crates.io/crates/nostr)
[![Documentation](https://docs.rs/nostr/badge.svg)](https://docs.rs/nostr)
[![Rustc Version 1.64.0+](https://img.shields.io/badge/rustc-1.64.0%2B-lightgrey.svg)](https://blog.rust-lang.org/2022/09/22/Rust-1.64.0.html)
[![CI](https://github.com/rust-nostr/nostr/actions/workflows/ci.yml/badge.svg)](https://github.com/rust-nostr/nostr/actions/workflows/ci.yml)
[![MIT](https://img.shields.io/crates/l/nostr.svg)](../../LICENSE)
![Lines of code](https://img.shields.io/tokei/lines/github/rust-nostr/nostr)

## Description

Rust implementation of Nostr protocol.

If you're writing a typical Nostr client or bot, you may be interested in [nostr-sdk](https://crates.io/crates/nostr-sdk).

## Getting started

```toml
[dependencies]
nostr = "0.22"
tungstenite = { version = "0.19", features = ["rustls-tls-webpki-roots"]}
```

NOTE: if you are using `bitcoin` v0.29, use version `nostr = "=0.22.0-bitcoin-v0.29"` insead.

```rust,no_run
use nostr::prelude::*;
use tungstenite::{Message as WsMessage};

fn main() -> Result<()> {
    // Generate new random keys
    let my_keys = Keys::generate();

    // or use your already existing
    //
    // From HEX or Bech32
    // let my_keys = Keys::from_sk_str("hex-or-bech32-secret-key")?;

    // Show bech32 public key
    let bech32_pubkey: String = my_keys.public_key().to_bech32()?;
    println!("Bech32 PubKey: {}", bech32_pubkey);

    let metadata = Metadata::new()
        .name("username")
        .display_name("My Username")
        .about("Description")
        .picture(Url::parse("https://example.com/avatar.png")?)
        .banner(Url::parse("https://example.com/banner.png")?)
        .nip05("username@example.com")
        .lud16("yuki@getalby.com");

    let event: Event = EventBuilder::set_metadata(metadata).to_event(&my_keys)?;

    // New text note
    let event: Event = EventBuilder::new_text_note("Hello from Nostr SDK", &[]).to_event(&my_keys)?;

    // New POW text note
    let event: Event = EventBuilder::new_text_note("My first POW text note from Nostr SDK", &[]).to_pow_event(&my_keys, 20)?;

    // Connect to relay
    let (mut socket, _) = tungstenite::connect("wss://relay.damus.io").expect("Can't connect to relay");

    // Send msg
    let msg = ClientMessage::new_event(event).as_json();
    socket.write_message(WsMessage::Text(msg)).expect("Impossible to send message");

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

## Crate Feature Flags

The following crate feature flags are available:

| Feature             | Default | Description                                                                                                                |
| ------------------- | :-----: | -------------------------------------------------------------------------------------------------------------------------- |
| `blocking`          |   No    | Needed to use `NIP-05` and `NIP-11` features in not async/await context                                                    |
| `vanity`            |   No    | Enable vanity public key mining module                                                                                     |
| `all-nips`          |   Yes   | Enable all NIPs                                                                                                            |
| `nip03`             |   No    | Enable NIP-03: OpenTimestamps Attestations for Events                                                                      |
| `nip04`             |   Yes   | Enable NIP-04: Encrypted Direct Message                                                                                    |
| `nip05`             |   Yes   | Enable NIP-05: Mapping Nostr keys to DNS-based internet identifiers                                                        |
| `nip06`             |   Yes   | Enable NIP-06: Basic key derivation from mnemonic seed phrase                                                              |
| `nip11`             |   Yes   | Enable NIP-11: Relay Information Document                                                                                  |
| `nip19`             |   Yes   | Enable NIP-19: bech32-encoded entities                                                                                     |
| `nip21`             |   Yes   | Enable NIP-21: `nostr` URI scheme                                                                                          |
| `nip46`             |   Yes   | Enable NIP-46: Nostr Connect                                                                                               |
| `nip47`             |   Yes   | Enable NIP-47: Nostr Wallet Connect                                                                                        |

## Supported NIPs

| Supported  | NIP                                                                                                                                |
|:----------:| ---------------------------------------------------------------------------------------------------------------------------------- |
| ✅         | [01 - Basic protocol flow description](https://github.com/nostr-protocol/nips/blob/master/01.md)                                    |
| ✅         | [02 - Contact List and Petnames](https://github.com/nostr-protocol/nips/blob/master/02.md)                                          |
| ✅         | [03 - OpenTimestamps Attestations for Events](https://github.com/nostr-protocol/nips/blob/master/03.md)                             |
| ✅         | [04 - Encrypted Direct Message](https://github.com/nostr-protocol/nips/blob/master/04.md)                                           |
| ✅         | [05 - Mapping Nostr keys to DNS-based internet identifiers](https://github.com/nostr-protocol/nips/blob/master/05.md)               |
| ✅         | [06 - Basic key derivation from mnemonic seed phrase](https://github.com/nostr-protocol/nips/blob/master/06.md)                     |
| ✅         | [09 - Event Deletion](https://github.com/nostr-protocol/nips/blob/master/09.md)                                                     |
| ✅         | [10 - Conventions for clients' use of `e` and `p` tags in text events](https://github.com/nostr-protocol/nips/blob/master/10.md)    |
| ✅         | [11 - Relay Information Document](https://github.com/nostr-protocol/nips/blob/master/11.md)                                         |
| ✅         | [12 - Generic Tag Queries](https://github.com/nostr-protocol/nips/blob/master/12.md)                                                |
| ✅         | [13 - Proof of Work](https://github.com/nostr-protocol/nips/blob/master/13.md)                                                      |
| ✅         | [14 - Subject tag in text events](https://github.com/nostr-protocol/nips/blob/master/14.md)                                         |
| ❌         | [15 - Nostr Marketplace](https://github.com/nostr-protocol/nips/blob/master/15.md)                                                  |
| ✅         | [16 - Event Treatment](https://github.com/nostr-protocol/nips/blob/master/16.md)                                                    |
| ✅         | [18 - Reposts](https://github.com/nostr-protocol/nips/blob/master/18.md)                                                            |
| ✅         | [19 - bech32-encoded entities](https://github.com/nostr-protocol/nips/blob/master/19.md)                                            |
| ✅         | [20 - Command Results](https://github.com/nostr-protocol/nips/blob/master/20.md)                                                    |
| ✅         | [21 - URI scheme](https://github.com/nostr-protocol/nips/blob/master/21.md)                                                         |
| ✅         | [23 - Long-form Content](https://github.com/nostr-protocol/nips/blob/master/23.md)                                                  |
| ✅         | [25 - Reactions](https://github.com/nostr-protocol/nips/blob/master/25.md)                                                          |
| ✅         | [26 - Delegated Event Signing](https://github.com/nostr-protocol/nips/blob/master/26.md)                                            |
| ❌         | [27 - Text Note References](https://github.com/nostr-protocol/nips/blob/master/27.md)                                               |
| ✅         | [28 - Public Chat](https://github.com/nostr-protocol/nips/blob/master/28.md)                                                        |
| ✅         | [33 - Parameterized Replaceable Events](https://github.com/nostr-protocol/nips/blob/master/33.md)                                   |
| ✅         | [36 - Sensitive Content](https://github.com/nostr-protocol/nips/blob/master/36.md)                                                  |
| ❌         | [39 - External Identities in Profiles](https://github.com/nostr-protocol/nips/blob/master/39.md)                                    |
| ✅         | [40 - Expiration Timestamp](https://github.com/nostr-protocol/nips/blob/master/40.md)                                               |
| ✅         | [42 - Authentication of clients to relays](https://github.com/nostr-protocol/nips/blob/master/42.md)                                |
| ✅         | [45 - Event Counts](https://github.com/nostr-protocol/nips/blob/master/45.md)                                                       |
| ✅         | [46 - Nostr Connect](https://github.com/nostr-protocol/nips/blob/master/46.md)                                                      |
| ✅         | [47 - Wallet Connect](https://github.com/nostr-protocol/nips/blob/master/47.md)                                                     |
| ✅         | [50 - Keywords filter](https://github.com/nostr-protocol/nips/blob/master/50.md)                                                    |
| ✅         | [51 - Lists](https://github.com/nostr-protocol/nips/blob/master/51.md)                                                              |
| ✅         | [56 - Reporting](https://github.com/nostr-protocol/nips/blob/master/56.md)                                                          |
| ✅         | [57 - Lightning Zaps](https://github.com/nostr-protocol/nips/blob/master/57.md)                                                     |
| ✅         | [58 - Badges](https://github.com/nostr-protocol/nips/blob/master/58.md)                                                             |
| ✅         | [65 - Relay List Metadata](https://github.com/nostr-protocol/nips/blob/master/65.md)                                                |
| ✅         | [78 - Arbitrary custom app data](https://github.com/nostr-protocol/nips/blob/master/78.md)                                          |
| ❌         | [94 - File Metadata](https://github.com/nostr-protocol/nips/blob/master/94.md)                                                      |

## State

**This library is in an ALPHA state**, things that are implemented generally work but the API will change in breaking ways.

## License

This project is distributed under the MIT software license - see the [LICENSE](../../LICENSE) file for details

## Donations

⚡ Tips: <https://getalby.com/p/yuki>

⚡ Lightning Address: yuki@getalby.com