# Nostr

[![crates.io](https://img.shields.io/crates/v/nostr.svg)](https://crates.io/crates/nostr)
[![Documentation](https://docs.rs/nostr/badge.svg)](https://docs.rs/nostr)
[![MIT](https://img.shields.io/crates/l/nostr.svg)](../../LICENSE)

## Description

Rust implementation of Nostr protocol.

## Getting started

```toml
[dependencies]
anyhow = "1"
nostr = "0.4"
tungstenite = { version = "0.17", features = ["rustls-tls-webpki-roots"]}
url = "2"
```

```rust,no_run
use std::str::FromStr;
use nostr::{Event, Metadata};
use nostr::key::{FromBech32, Keys};
use nostr::message::ClientMessage;
use tungstenite::{Message as WsMessage};
use url::Url;

fn main() -> anyhow::Result<()> {
    // Generate new random keys
    let my_new_keys = Keys::generate_from_os_random();

    // Use your already existing bec32 keys
    let my_bech32_keys = Keys::from_bech32("nsec1...")?;

    // Use your already existing keys
    let my_keys = Keys::from_str("hex-secret-key")?;

    let metadata = Metadata::new()
        .name("username")
        .display_name("My Username")
        .about("Description")
        .picture(Url::from_str("https://example.com/avatar.png")?)
        .nip05("username@example.com");

    let event = Event::set_metadata(&my_keys, metadata)?;

    // Connect to relay
    let (mut socket, _) = tungstenite::connect(Url::parse("wss://relay.damus.io")?).expect("Can't connect to relay");

    // Send msg
    let msg = ClientMessage::new_event(event).to_json();
    socket.write_message(WsMessage::Text(msg))?;

    Ok(())
}
```

More examples can be found in the [examples](https://github.com/yukibtc/nostr-rs-sdk/tree/master/crates/nostr/examples) directory.

## NIPs

| Supported  | NIP                                                                                                                                |
|:----------:| ---------------------------------------------------------------------------------------------------------------------------------- |
| ✅         | [01 - Basic protocol flow description](https://github.com/nostr-protocol/nips/blob/master/01.md)                                    |
| ✅         | [02 - Contact List and Petnames](https://github.com/nostr-protocol/nips/blob/master/02.md)                                          |
| ❌         | [03 - OpenTimestamps Attestations for Events](https://github.com/nostr-protocol/nips/blob/master/03.md)                             |
| ✅         | [04 - Encrypted Direct Message](https://github.com/nostr-protocol/nips/blob/master/04.md)                                           |
| ✅         | [05 - Mapping Nostr keys to DNS-based internet identifiers](https://github.com/nostr-protocol/nips/blob/master/05.md)               |
| ✅         | [06 - Basic key derivation from mnemonic seed phrase](https://github.com/nostr-protocol/nips/blob/master/06.md)                     |
| ❌         | [08 - Handling Mentions](https://github.com/nostr-protocol/nips/blob/master/08.md)                                                  |
| ✅         | [09 - Event Deletion](https://github.com/nostr-protocol/nips/blob/master/09.md)                                                     |
| ❌         | [10 - Conventions for clients' use of `e` and `p` tags in text events](https://github.com/nostr-protocol/nips/blob/master/10.md)    |
| ✅         | [11 - Relay Information Document](https://github.com/nostr-protocol/nips/blob/master/11.md)                                         |
| ❌         | [12 - Generic Tag Queries](https://github.com/nostr-protocol/nips/blob/master/12.md)                                                |
| ✅         | [13 - Proof of Work](https://github.com/nostr-protocol/nips/blob/master/13.md)                                                      |
| ❌         | [14 - Subject tag in text events](https://github.com/nostr-protocol/nips/blob/master/14.md)                                         |
| ✅         | [15 - End of Stored Events Notice](https://github.com/nostr-protocol/nips/blob/master/15.md)                                        |
| ❌         | [16 - Event Treatment](https://github.com/nostr-protocol/nips/blob/master/16.md)                                                    |
| ❌         | [22 - Event created_at Limits](https://github.com/nostr-protocol/nips/blob/master/22.md)                                            |
| ✅         | [25 - Reactions](https://github.com/nostr-protocol/nips/blob/master/25.md)                                                          |
| ✅         | [28 - Public Chat](https://github.com/nostr-protocol/nips/blob/master/28.md)                                                        |

## State

**This library is in an ALPHA state**, things that are implemented generally work but the API will change in breaking ways.

## License

This project is distributed under the MIT software license - see the [LICENSE](../../LICENSE) file for details