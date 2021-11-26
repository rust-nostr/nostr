use nostr::{gen_keys, Event, RelayMessage};
use tungstenite::{connect, Message as WsMessage};
use url::Url;

const ALICE_SK: &str = "6b911fd37cdf5c81d4c0adb1ab7fa822ed253ab0ad9aa18d77257c88b29b718e";
const BOB_SK: &str = "7b911fd37cdf5c81d4c0adb1ab7fa822ed253ab0ad9aa18d77257c88b29b718e";

fn main() {
    env_logger::init();

    let (mut socket, response) =
        connect(Url::parse("ws://localhost:3333/ws").unwrap()).expect("Can't connect");

    println!("Connected to the server");
    println!("Response HTTP code: {}", response.status());
    println!("Response contains the following headers:");
    for (ref header, _value) in response.headers() {
        println!("* {}", header);
    }

    let (alice_keypair, alice_pubkey, _) = gen_keys(ALICE_SK);
    let (bob_keypair, bob_pubkey, _) = gen_keys(BOB_SK);

    let alice_says_hi = Event::new_textnote("hi from alice", &alice_keypair)
        .unwrap()
        .as_json();
    let bob_says_hi = Event::new_textnote("bob says hello", &bob_keypair)
        .unwrap()
        .as_json();

    let subscribe_to_alice = format!("sub-key:{}", alice_pubkey);
    let subscribe_to_bob = format!("sub-key:{}", bob_pubkey);

    socket
        .write_message(WsMessage::Text(subscribe_to_alice.into()))
        .unwrap();

    socket
        .write_message(WsMessage::Text(subscribe_to_bob.into()))
        .unwrap();

    socket
        .write_message(WsMessage::Text(alice_says_hi))
        .unwrap();
    socket.write_message(WsMessage::Text(bob_says_hi)).unwrap();

    loop {
        let msg = socket.read_message().expect("Error reading message");
        let msg_text = msg.to_text().expect("Failed to conver message to text");
        let handled_message = RelayMessage::from_json(msg_text).expect("Failed to handle message");
        match handled_message {
            // Message::Empty => {
            //     println!("Got an empty message... why?");
            // }
            // Message::Ping => {
            //     println!("Got PING, sending PONG");
            //     socket
            //         .write_message(WsMessage::Text("PONG".into()))
            //         .unwrap();
            // }
            RelayMessage::Notice { message } => {
                println!("Got a notice: {}", message);
            }
            RelayMessage::Event {
                event,
                subscription_id: _,
            } => {
                println!("Got an event!");
                dbg!(event);
            }
        }
    }
}
