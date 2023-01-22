# Nostr

[![crates.io](https://img.shields.io/crates/v/nostr.svg)](https://crates.io/crates/nostr)
[![crates.io - Downloads](https://img.shields.io/crates/d/nostr)](https://crates.io/crates/nostr)
[![Documentation](https://docs.rs/nostr/badge.svg)](https://docs.rs/nostr)
[![CI](https://github.com/yukibtc/nostr-rs-sdk/actions/workflows/ci.yml/badge.svg)](https://github.com/yukibtc/nostr-rs-sdk/actions/workflows/ci.yml)
[![MIT](https://img.shields.io/crates/l/nostr.svg)](../../LICENSE)

## Description

Rust implementation of Nostr protocol.

## Getting started

```toml
[dependencies]
nostr = "0.14"
tungstenite = { version = "0.18", features = ["rustls-tls-webpki-roots"]}
```

```rust,no_run
use nostr::{Event, EventBuilder, Metadata, Keys, Result};
use nostr::message::ClientMessage;
use tungstenite::{Message as WsMessage};

fn main() -> Result<()> {
    // Generate new random keys
    let my_keys = Keys::generate_from_os_random();

    // or use your already existing
    //
    // From HEX or Bech32
    // use nostr::key::FromSkStr;
    // let my_keys = Keys::from_sk_str("hex-or-bech32-secret-key")?;

    // Show bech32 public key
    use nostr::util::nips::nip19::ToBech32;
    let bech32_pubkey: String = my_keys.public_key().to_bech32()?;
    println!("Bech32 PubKey: {}", bech32_pubkey);

    let metadata = Metadata::new()
        .name("username")
        .display_name("My Username")
        .about("Description")
        .picture("https://example.com/avatar.png")
        .banner("https://example.com/banner.png")
        .nip05("username@example.com")
        .lud16("yuki@stacker.news");

    let event: Event = EventBuilder::set_metadata(metadata)?.to_event(&my_keys)?;

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

More examples can be found in the [examples](https://github.com/yukibtc/nostr-rs-sdk/tree/master/crates/nostr/examples) directory.

## Crate Feature Flags

The following crate feature flags are available:

| Feature             | Default | Description                                                                                                                |
| ------------------- | :-----: | -------------------------------------------------------------------------------------------------------------------------- |
| `full`              |   Yes   | Enable all main features: `all-nips` and `event`                                                                           |
| `blocking`          |   No    | Needed to use `NIP-05` and `NIP-11`features in not async/await context                                                     |
| `event`             |   Yes   | Enable `event`, `message` and `types` modules                                                                              |
| `all-nips`          |   Yes   | Enable all NIPs                                                                                                            |
| `nip04`             |   Yes   | Enable NIP-04: Encrypted Direct Message                                                                                    |
| `nip05`             |   Yes   | Enable NIP-05: Mapping Nostr keys to DNS-based internet identifiers                                                        |
| `nip06`             |   Yes   | Enable NIP-06: Basic key derivation from mnemonic seed phrase                                                              |
| `nip11`             |   Yes   | Enable NIP-11: Relay Information Document                                                                                  |
| `nip13`             |   Yes   | Enable NIP-13: Proof of Work                                                                                               |
| `nip19`             |   Yes   | Enable NIP-19: bech32-encoded entities                                                                                     |
| `nip26`             |   Yes   | Enable NIP-26: Delegated Event Signing                                                                                     |

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
| ✅         | [15 - End of Stored Events Notice](https://github.com/nostr-protocol/nips/blob/master/15.md)                                        |
| ✅         | [16 - Event Treatment](https://github.com/nostr-protocol/nips/blob/master/16.md)                                                    |
| ✅         | [18 - Reposts](https://github.com/nostr-protocol/nips/blob/master/18.md)                                                            |
| ✅         | [19 - bech32-encoded entities](https://github.com/nostr-protocol/nips/blob/master/19.md)                                            |
| ✅         | [20 - Command Results](https://github.com/nostr-protocol/nips/blob/master/20.md)                                                    |
| ✅         | [25 - Reactions](https://github.com/nostr-protocol/nips/blob/master/25.md)                                                          |
| ✅         | [26 - Delegated Event Signing](https://github.com/nostr-protocol/nips/blob/master/26.md)                                            |
| ✅         | [28 - Public Chat](https://github.com/nostr-protocol/nips/blob/master/28.md)                                                        |
| ✅         | [36 - Sensitive Content](https://github.com/nostr-protocol/nips/blob/master/36.md)                                                  |
| ✅         | [40 - Expiration Timestamp](https://github.com/nostr-protocol/nips/blob/master/40.md)                                               |
| ✅         | [42 - Authentication of clients to relays](https://github.com/nostr-protocol/nips/blob/master/42.md)                                |

## State

**This library is in an ALPHA state**, things that are implemented generally work but the API will change in breaking ways.

## License

This project is distributed under the MIT software license - see the [LICENSE](../../LICENSE) file for details

## Donations

⚡ Tips: https://getalby.com/p/yuki

⚡ Lightning Address: yuki@getalby.com