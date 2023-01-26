// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use nostr::key::Keys;
use nostr::{ClientMessage, Event, EventBuilder, Metadata, Result, Url};
use tungstenite::{connect, Message as WsMessage};

const WS_ENDPOINT: &str = "wss://relay.damus.io";

fn main() -> Result<()> {
    env_logger::init();

    let (mut socket, _response) = connect(WS_ENDPOINT).expect("Can't connect to relay");

    let my_keys = Keys::generate();

    let metadata = Metadata::new()
        .name("username")
        .display_name("My Username")
        .about("Description")
        .picture(Url::parse("https://example.com/avatar.png")?)
        .banner(Url::parse("https://example.com/banner.png")?)
        .nip05("username@example.com")
        .lud16("yuki@stacker.news");

    let event: Event = EventBuilder::set_metadata(metadata)?.to_event(&my_keys)?;

    socket
        .write_message(WsMessage::Text(ClientMessage::new_event(event).as_json()))
        .unwrap();

    Ok(())
}
