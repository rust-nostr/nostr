# Nostr

[![crates.io](https://img.shields.io/crates/v/nostr-sdk-base.svg)](https://crates.io/crates/nostr-sdk-base)
[![Documentation](https://docs.rs/nostr-sdk-base/badge.svg)](https://docs.rs/nostr-sdk-base)
[![MIT](https://img.shields.io/crates/l/nostr-sdk-base.svg)](../../LICENSE)

## Description

Rust implementation of Nostr protocol.

## Getting started

```toml
[dependencies]
anyhow = "1"
nostr-sdk-base = "0.1"
tungstenite = { version = "0.17", features = ["rustls-tls-webpki-roots"]}
```

```rust,no_run
use std::str::FromStr;
use nostr_sdk_base::key::{FromBech32, Keys};
use nostr_sdk_base::message::ClientMessage;
use tungstenite::{Message as WsMessage};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Generate new random keys
    let my_new_keys = Keys::generate_from_os_random();

    // Use your already existing bec32 keys
    let my_bech32_keys = Keys::from_bech32("nsec1...")?;

    // Use your already existing keys
    let my_keys = Keys::from_str("hex-secret-key")?;

    let event = Event::set_metadata(
        &my_keys,
        Some("nostr_sdk_base"),
        Some("Nostr SDK"),
        Some("Description"),
        Some("https://example.com/avatar.png"),
    )?;

    // Connect to relay
    let (mut socket, _) = tungstenite::connect(Url::parse("wss://relay.damus.io")?).expect("Can't connect to relay");

    // Send msg
    let msg = ClientMessage::new_event(event).to_json();
    socket.write_message(WsMessage::Text(msg))?;

    Ok(())
}
```

More examples can be found in the [examples](https://github.com/yukibtc/nostr-rs-sdk/tree/master/crates/nostr-sdk-base/examples) directory.

## State

**This library is in an ALPHA state**, things that are implemented generally work but the API will change in breaking ways.

## License

This project is distributed under the MIT software license - see the [LICENSE](../../LICENSE) file for details