// Copyright (c) 2021 Paul Miller
// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::str::FromStr;

use nostr::event::{Kind, KindBase};
use nostr::secp256k1::SecretKey;
use nostr::url::Url;
use nostr::{ClientMessage, EventBuilder, Keys, RelayMessage, Result, SubscriptionFilter};
use tungstenite::{connect, Message as WsMessage};

const ALICE_SK: &str = "6b911fd37cdf5c81d4c0adb1ab7fa822ed253ab0ad9aa18d77257c88b29b718e";
const BOB_SK: &str = "7b911fd37cdf5c81d4c0adb1ab7fa822ed253ab0ad9aa18d77257c88b29b718e";
// const WS_ENDPOINT: &str = "wss://relayer.fiatjaf.com/";
const WS_ENDPOINT: &str = "wss://nostr-relay-dev.wlvs.space";

fn main() -> Result<()> {
    env_logger::init();

    let (mut socket, response) = connect(Url::parse(WS_ENDPOINT)?).expect("Can't connect");

    println!("Connected to the server");
    println!("Response HTTP code: {}", response.status());
    println!("Response contains the following headers:");
    for (ref header, _value) in response.headers() {
        println!("* {}", header);
    }

    let alice_keys = Keys::new(SecretKey::from_str(ALICE_SK)?);

    let bob_keys = Keys::new(SecretKey::from_str(BOB_SK)?);

    let alice_says_hi = ClientMessage::new_event(
        EventBuilder::new_text_note("hi from alice", &vec![]).to_event(&alice_keys)?,
    );
    let bob_says_hi = ClientMessage::new_event(
        EventBuilder::new_text_note("bob says hello", &vec![]).to_event(&bob_keys)?,
    );

    let subscribe_to_alice = ClientMessage::new_req(
        "abcdefgh",
        vec![SubscriptionFilter::new()
            .authors(vec![alice_keys.public_key()])
            .kind(Kind::Base(KindBase::TextNote))],
    );

    let subscribe_to_bob = ClientMessage::new_req(
        "1234567",
        vec![SubscriptionFilter::new()
            .authors(vec![bob_keys.public_key()])
            .kind(Kind::Base(KindBase::TextNote))],
    );

    socket.write_message(WsMessage::Text(subscribe_to_alice.as_json()))?;
    socket.write_message(WsMessage::Text(subscribe_to_bob.as_json()))?;

    socket.write_message(WsMessage::Text(alice_says_hi.as_json()))?;
    socket.write_message(WsMessage::Text(bob_says_hi.as_json()))?;

    loop {
        let msg = socket.read_message().expect("Error reading message");
        let msg_text = msg.to_text().expect("Failed to conver message to text");
        if let Ok(handled_message) = RelayMessage::from_json(msg_text) {
            match handled_message {
                RelayMessage::Notice { message } => {
                    println!("Got a notice: {}", message);
                }
                RelayMessage::Event {
                    event: _,
                    subscription_id: _,
                } => {
                    println!("Got an event!");
                }
                RelayMessage::EndOfStoredEvents { subscription_id: _ } => {
                    println!("Relay signalled End of Stored Events");
                }
                RelayMessage::Ok {
                    event_id,
                    status,
                    message,
                } => {
                    println!("Got OK message: {} - {} - {}", event_id, status, message);
                }
                RelayMessage::Empty => {
                    println!("Empty message");
                }
            }
        } else {
            println!("Got unexpected message: {}", msg_text);
        }
    }
}
