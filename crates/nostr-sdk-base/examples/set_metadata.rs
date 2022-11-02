// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::error::Error;

use nostr_sdk_base::{ClientMessage, Event, Keys};
use tungstenite::{connect, Message as WsMessage};
use url::Url;

const MY_BECH32_SK: &str = "nsec1...";

const WS_ENDPOINT: &str = "wss://relay.damus.io";

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let (mut socket, _response) =
        connect(Url::parse(WS_ENDPOINT)?).expect("Can't connect to relay");

    let my_keys = Keys::new_from_bech32(MY_BECH32_SK).unwrap();

    let event = Event::set_metadata(
        &my_keys,
        "username",
        "Username",
        Some("Description"),
        Some("https://example.com/avatar.png"),
    )?;

    socket.write_message(WsMessage::Text(ClientMessage::new_event(event).to_json()))?;

    Ok(())
}
