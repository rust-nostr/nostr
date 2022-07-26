use std::error::Error;

use nostr::{ClientMessage, Event, Keys, Kind, RelayMessage, SubscriptionFilter};
use tungstenite::{connect, Message as WsMessage};
use url::Url;

const ALICE_SK: &str = "6b911fd37cdf5c81d4c0adb1ab7fa822ed253ab0ad9aa18d77257c88b29b718e";
const BOB_SK: &str = "7b911fd37cdf5c81d4c0adb1ab7fa822ed253ab0ad9aa18d77257c88b29b718e";
const WS_ENDPOINT: &str = "wss://relayer.fiatjaf.com/";

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let (mut socket, response) = connect(Url::parse(WS_ENDPOINT)?).expect("Can't connect");

    println!("Connected to the server");
    println!("Response HTTP code: {}", response.status());
    println!("Response contains the following headers:");
    for (ref header, _value) in response.headers() {
        println!("* {}", header);
    }

    let alice_keys = Keys::new(ALICE_SK)?;

    let bob_keys = Keys::new(BOB_SK)?;

    let alice_says_hi =
        ClientMessage::new_event(Event::new_textnote("hi from alice", &alice_keys)?);
    let bob_says_hi = ClientMessage::new_event(Event::new_textnote("bob says hello", &bob_keys)?);

    let subscribe_to_alice = ClientMessage::new_req(
        "abcdefgh",
        vec![SubscriptionFilter::new()
            .authors(vec![alice_keys.public_key])
            .kind(Kind::TextNote)],
    );

    let subscribe_to_bob = ClientMessage::new_req(
        "1234567",
        vec![SubscriptionFilter::new()
            .authors(vec![bob_keys.public_key])
            .kind(Kind::TextNote)],
    );

    socket.write_message(WsMessage::Text(subscribe_to_alice.to_json()))?;
    socket.write_message(WsMessage::Text(subscribe_to_bob.to_json()))?;

    socket.write_message(WsMessage::Text(alice_says_hi.to_json()))?;
    socket.write_message(WsMessage::Text(bob_says_hi.to_json()))?;

    loop {
        let msg = socket.read_message().expect("Error reading message");
        let msg_text = msg.to_text().expect("Failed to conver message to text");
        let handled_message = RelayMessage::from_json(msg_text).expect("Failed to handle message");
        match handled_message {
            RelayMessage::Empty => {
                println!("Empty message")
            }
            RelayMessage::EndOfStoredEvents { subscription_id } => {
                println!("End of stored events for subscription {}", subscription_id);
            }
            RelayMessage::Notice { message } => {
                println!("Got a notice: {}", message);
            }
            RelayMessage::Event {
                event: _,
                subscription_id: _,
            } => {
                println!("Got an event!");
            }
        }
    }
}
