// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::error::Error;

use nostr::{ClientMessage, Event, Keys};
use tungstenite::{connect, Message as WsMessage};
use url::Url;

const MY_BECH32_SK: &str = "nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85";

const WS_ENDPOINT: &str = "wss://relay.damus.io";

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let (mut socket, _response) =
        connect(Url::parse(WS_ENDPOINT)?).expect("Can't connect to relay");

    let my_keys = Keys::new_from_bech32(MY_BECH32_SK).unwrap();

    let event = Event::delete(
        &my_keys,
        vec!["14689524662bccd0835e87aa978869228e3605db4c5d30f275f9427f7e0996d5"],
        "these posts were published by accident",
    )?;

    socket.write_message(WsMessage::Text(ClientMessage::new_event(event).to_json()))?;

    Ok(())
}
