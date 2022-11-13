// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::error::Error;

use nostr::key::Keys;
use nostr::{ClientMessage, Event};
use tungstenite::{connect, Message as WsMessage};
use url::Url;

const WS_ENDPOINT: &str = "wss://relay.damus.io";

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let (mut socket, _response) =
        connect(Url::parse(WS_ENDPOINT)?).expect("Can't connect to relay");

    let my_keys = Keys::generate_from_os_random();

    let event = Event::set_metadata(
        &my_keys,
        Some("username"),
        Some("Username"),
        Some("Description"),
        Some("https://example.com/avatar.png"),
    )?;

    socket.write_message(WsMessage::Text(ClientMessage::new_event(event).to_json()))?;

    Ok(())
}
