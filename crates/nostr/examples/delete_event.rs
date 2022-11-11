// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::error::Error;
use std::str::FromStr;

use bitcoin_hashes::sha256::Hash;
use nostr::key::{FromBech32, Keys};
use nostr::{ClientMessage, Event};
use tungstenite::{connect, Message as WsMessage};
use url::Url;

const MY_BECH32_SK: &str = "nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85";

const WS_ENDPOINT: &str = "wss://relay.damus.io";

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let (mut socket, _response) =
        connect(Url::parse(WS_ENDPOINT)?).expect("Can't connect to relay");

    let my_keys = Keys::from_bech32(MY_BECH32_SK).unwrap();

    let event_id =
        Hash::from_str("7469af3be8c8e06e1b50ef1caceba30392ddc0b6614507398b7d7daa4c218e96")?;
    let event = Event::delete(
        &my_keys,
        vec![event_id],
        Some("these posts were published by accident"),
    )?;

    socket.write_message(WsMessage::Text(ClientMessage::new_event(event).to_json()))?;

    Ok(())
}
