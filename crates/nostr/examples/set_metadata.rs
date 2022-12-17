// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use nostr::key::Keys;
use nostr::url::Url;
use nostr::{ClientMessage, Event, EventBuilder, Metadata, Result};
use tungstenite::{connect, Message as WsMessage};

const WS_ENDPOINT: &str = "wss://relay.damus.io";

fn main() -> Result<()> {
    env_logger::init();

    let (mut socket, _response) =
        connect(Url::parse(WS_ENDPOINT)?).expect("Can't connect to relay");

    let my_keys = Keys::generate_from_os_random();

    let metadata = Metadata::new()
        .name("username")
        .display_name("My Username")
        .about("Description")
        .picture(Url::parse("https://example.com/avatar.png")?)
        .nip05("username@example.com");

    let event: Event = EventBuilder::set_metadata(&my_keys, metadata)?.to_event(&my_keys)?;

    socket
        .write_message(WsMessage::Text(ClientMessage::new_event(event).to_json()))
        .unwrap();

    Ok(())
}
