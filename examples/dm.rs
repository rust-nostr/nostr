use nostr::{
    gen_keys, util::nip04::decrypt, ClientMessage, Event, Kind, RelayMessage, SubscriptionFilter,
};
use std::{thread, time};
use tungstenite::{connect, Message as WsMessage};
use url::Url;

const ALICE_SK: &str = "6b911fd37cdf5c81d4c0adb1ab7fa822ed253ab0ad9aa18d77257c88b29b718e";
const BOB_SK: &str = "7b911fd37cdf5c81d4c0adb1ab7fa822ed253ab0ad9aa18d77257c88b29b718e";
// const WS_ENDPOINT: &str = "wss://nostr-relay.freeberty.net";
const WS_ENDPOINT: &str = "wss://relayer.fiatjaf.com/";
// const LOCAL_WS_ENDPOINT: &str = "ws://localhost:3333/ws";

fn main() {
    env_logger::init();

    let (mut socket, _response) =
        connect(Url::parse(WS_ENDPOINT).unwrap()).expect("Can't connect to Bob's relay");

    let (_alice_keypair, alice_pubkey, alice_sk) = gen_keys(ALICE_SK);
    let (_bob_keypair, bob_pubkey, bob_sk) = gen_keys(BOB_SK);

    let alice_to_bob = "Hey bob this is alice (ping)";
    let bob_to_alice = "Hey alice this is bob (pong)";

    let alice_encrypted_msg =
        Event::new_encrypted_direct_msg(alice_sk, &bob_pubkey, alice_to_bob.clone());

    // TODO extract this logic into Message
    // let subscribe_to_alice = format!(
    //     "[\"REQ\", \"abcdefg\", {{ \"authors\": [\"{}\"]}}]",
    //     alice_pubkey
    // );
    let subscribe_to_alice =
        ClientMessage::new_req("abcdefg", SubscriptionFilter::new(vec![alice_pubkey])).to_json();
    dbg!(subscribe_to_alice.clone());

    let subscribe_to_bob =
        ClientMessage::new_req("123456", SubscriptionFilter::new(vec![bob_pubkey])).to_json();
    //  format!(
    //     "[\"REQ\", \"123456\", {{ \"authors\": [\"{}\"]}}]",
    //     bob_pubkey
    // );

    socket
        .write_message(WsMessage::Text(subscribe_to_alice.into()))
        .unwrap();

    socket
        .write_message(WsMessage::Text(subscribe_to_bob.into()))
        .unwrap();

    socket
        .write_message(WsMessage::Text(
            ClientMessage::new_event(alice_encrypted_msg).to_json(),
        ))
        .unwrap();

    loop {
        let msg = socket.read_message().expect("Error reading message");
        let msg_text = msg.to_text().expect("Failed to conver message to text");
        let handled_message = RelayMessage::from_json(msg_text).expect("Failed to handle message");
        match handled_message {
            // RelayMessage::Empty => {
            //     println!("Got an empty message... why?");
            // }
            // RelayMessage::Ping => {
            //     println!("Got PING, sending PONG");
            //     socket
            //         .write_message(WsMessage::Text("PONG".into()))
            //         .unwrap();
            // }
            RelayMessage::Empty => {
                println!("Empty message (heartbeat I guess?)")
            }
            RelayMessage::Notice { message } => {
                println!("Got a notice: {}", message);
            }
            RelayMessage::Event {
                event,
                subscription_id: _,
            } => {
                if event.kind == Kind::EncryptedDirectMessage {
                    println!("it's a dm");

                    if event.tags[0].content() == &alice_pubkey.to_string() {
                        println!("It's to alice!");
                        println!("Encrypted it says {}", event.content);
                        println!(
                            "Decrypted it says {}",
                            decrypt(&alice_sk, &bob_pubkey, &event.content).unwrap()
                        );
                        thread::sleep(time::Duration::from_millis(5000));
                        let alice_encrypted_msg = Event::new_encrypted_direct_msg(
                            alice_sk,
                            &bob_pubkey,
                            alice_to_bob.clone(),
                        );
                        socket
                            .write_message(WsMessage::Text(
                                ClientMessage::new_event(alice_encrypted_msg).to_json(),
                            ))
                            .unwrap();
                    } else if event.tags[0].content() == &bob_pubkey.to_string() {
                        println!("It's to bob!");
                        println!("Encrypted it says {}", event.content);
                        println!(
                            "Decrypted it says {}",
                            decrypt(&alice_sk, &bob_pubkey, &event.content).unwrap()
                        );
                        thread::sleep(time::Duration::from_millis(5000));
                        let bob_encrypted_msg = Event::new_encrypted_direct_msg(
                            bob_sk,
                            &alice_pubkey,
                            bob_to_alice.clone(),
                        );
                        socket
                            .write_message(WsMessage::Text(
                                ClientMessage::new_event(bob_encrypted_msg).to_json(),
                            ))
                            .unwrap();
                    }
                } else {
                    println!("it's not a dm");
                    dbg!(event);
                }
            }
        }
    }
}
