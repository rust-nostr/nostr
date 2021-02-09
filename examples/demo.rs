use secp256k1::{rand::rngs::OsRng, schnorrsig, Secp256k1};
use tungstenite::{connect, Message as WsMessage};

use url::Url;

use nostr::{Event, Message};

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

    let secp = Secp256k1::new();
    let mut rng = OsRng::new().expect("OsRng");
    let keypair = schnorrsig::KeyPair::new(&secp, &mut rng);
    let pubkey = schnorrsig::PublicKey::from_keypair(&secp, &keypair).to_string();

    let event = Event::new_textnote("hello", &keypair);
    let event_json = event.as_json();
    dbg!(event_json.clone());

    let sub_message = format!("sub-key:{}", pubkey);
    dbg!(sub_message.clone());

    socket
        .write_message(WsMessage::Text(sub_message.into()))
        .unwrap();

    loop {
        let msg = socket.read_message().expect("Error reading message");
        let msg_text = msg.to_text().expect("Failed to conver message to text");
        let handled_message = Message::handle(msg_text).expect("Failed to handle message");
        match handled_message {
            Message::Empty => {
                println!("Got an empty message... why?");
                // Since I get these empty messagues on a regular basis
                // it seems a good place to send out my test message.
                socket
                    .write_message(WsMessage::Text(event_json.clone()))
                    .unwrap();
            }
            Message::Ping => {
                println!("Got PING, sending PONG");
                socket
                    .write_message(WsMessage::Text("PONG".into()))
                    .unwrap();
            }
            Message::Notice(notice) => {
                println!("Got a notice: {}", notice);
            }
            Message::Event(event) => {
                println!("Got an event!");
                dbg!(event);
            }
        }
    }
}
