// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::str::FromStr;

use nostr::key::Keys;
use nostr::{ClientMessage, Event, Metadata};
use tungstenite::{connect, Message as WsMessage};
use url::Url;

const WS_ENDPOINT: &str = "wss://relay.damus.io";

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let (mut socket, _response) =
        connect(Url::parse(WS_ENDPOINT)?).expect("Can't connect to relay");

    let my_keys = Keys::generate_from_os_random();

    let metadata = Metadata::new()
        .name("username")
        .display_name("My Username")
        .about("Description")
        .picture(Url::from_str("https://example.com/avatar.png")?)
        .nip05("username@example.com");

    let event = Event::set_metadata(&my_keys, metadata)?;

    socket.write_message(WsMessage::Text(ClientMessage::new_event(event).to_json()))?;

    Ok(())
}
