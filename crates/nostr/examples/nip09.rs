// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::str::FromStr;

use nostr::secp256k1::SecretKey;
use nostr::url::Url;
use nostr::util::nips::nip19::FromBech32;
use nostr::{ClientMessage, Event, EventBuilder, Keys, Result, Sha256Hash};
use tungstenite::{connect, Message as WsMessage};

const MY_BECH32_SK: &str = "nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85";

const WS_ENDPOINT: &str = "wss://relay.damus.io";

fn main() -> Result<()> {
    env_logger::init();

    let (mut socket, _response) =
        connect(Url::parse(WS_ENDPOINT)?).expect("Can't connect to relay");

    let secret_key = SecretKey::from_bech32(MY_BECH32_SK)?;
    let my_keys = Keys::new(secret_key);

    let event_id =
        Sha256Hash::from_str("7469af3be8c8e06e1b50ef1caceba30392ddc0b6614507398b7d7daa4c218e96")?;

    let event: Event = EventBuilder::delete(
        vec![event_id],
        Some("these posts were published by accident"),
    )
    .to_event(&my_keys)?;

    socket.write_message(WsMessage::Text(ClientMessage::new_event(event).as_json()))?;

    Ok(())
}
