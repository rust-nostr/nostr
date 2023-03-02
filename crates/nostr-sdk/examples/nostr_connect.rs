// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use nostr::nips::nip46::{Message, Request};
use nostr_sdk::prelude::*;

const APP_SECRET_KEY: &str = "nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85";

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let secret_key = SecretKey::from_bech32(APP_SECRET_KEY)?;
    let app_keys = Keys::new(secret_key);

    let nostr_connect_uri: NostrConnectURI = NostrConnectURI::new(
        app_keys.public_key(),
        Url::parse("ws://192.168.7.233:7777")?,
        "Nostr SDK",
    )
    .url(Url::parse("https://example.com")?);

    let opts = Options::new().wait_for_send(true);

    let client = Client::new_with_opts(&app_keys, opts);
    client.add_relay("ws://192.168.7.233:7777", None).await?;

    println!("\n###############################################\n");
    println!("Nostr Connect URI: {nostr_connect_uri}");
    println!("\n###############################################\n");

    client.connect().await;

    // Listen for connect ACK
    let signer_pubkey = get_signer_pubkey(&client).await;
    println!("Received signer pubkey: {signer_pubkey}");

    println!("\n###############################################\n");

    let msg = Message::request(Request::GetPublicKey);
    let res = get_response(&client, signer_pubkey, msg).await?;
    if let Response::GetPublicKey(pubkey) = res {
        println!("Received pubeky {pubkey}");
        println!("\n###############################################\n");
    }

    // compose unsigned event
    let unsigned_event = EventBuilder::new_text_note("Hello world from Nostr SDK", &[])
        .to_unsigned_event(signer_pubkey);
    let msg = Message::request(Request::SignEvent(unsigned_event.clone()));
    let res = get_response(&client, signer_pubkey, msg).await?;
    if let Response::SignEvent(sig) = res {
        let event = unsigned_event.add_signature(sig)?;
        let id = client.send_event(event).await?;
        println!("Published event {id}");
        println!("\n###############################################\n");
    }

    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("generic error")]
    Generic,
    #[error("response error: {0}")]
    Response(String),
    #[error(transparent)]
    Keys(#[from] nostr_sdk::nostr::key::Error),
    #[error(transparent)]
    Builder(#[from] nostr_sdk::nostr::event::builder::Error),
    #[error(transparent)]
    Client(#[from] nostr_sdk::client::Error),
    #[error(transparent)]
    Nip46(#[from] nostr_sdk::nostr::nips::nip46::Error),
    #[error(transparent)]
    JSON(#[from] serde_json::Error),
}

async fn get_response(
    client: &Client,
    signer_pubkey: XOnlyPublicKey,
    msg: Message,
) -> Result<Response, Error> {
    let keys = client.keys();
    let req_id = msg.id();
    let req = msg.to_request()?;

    let event = EventBuilder::nostr_connect(&keys, signer_pubkey, msg)?.to_event(&keys)?;
    client.send_event(event).await?;

    client
        .subscribe(vec![Filter::new()
            .pubkey(keys.public_key())
            .kind(Kind::NostrConnect)
            .since(Timestamp::now())])
        .await;

    let mut notifications = client.notifications();
    while let Ok(notification) = notifications.recv().await {
        if let RelayPoolNotification::Event(_url, event) = notification {
            if event.kind == Kind::NostrConnect {
                match decrypt(&keys.secret_key()?, &event.pubkey, &event.content) {
                    Ok(msg) => {
                        let msg = Message::from_json(msg)?;

                        println!("New message received: {msg:#?}");
                        println!("\n###############################################\n");

                        if let Message::Response { id, result, error } = &msg {
                            if &req_id == id {
                                if let Some(result) = result {
                                    let res = match req {
                                        Request::SignEvent(_) => {
                                            let sig = serde_json::from_value(result.to_owned())?;
                                            Response::SignEvent(sig)
                                        }
                                        Request::GetPublicKey => {
                                            let pubkey = serde_json::from_value(result.to_owned())?;
                                            Response::GetPublicKey(pubkey)
                                        }
                                        _ => todo!(),
                                    };
                                    client.unsubscribe().await;
                                    return Ok(res);
                                }

                                if let Some(error) = error {
                                    client.unsubscribe().await;
                                    return Err(Error::Response(error.to_owned()));
                                }

                                break;
                            }
                        }
                    }
                    Err(e) => eprintln!("Impossible to decrypt NIP46 message: {e}"),
                }
            }
        }
    }

    client.unsubscribe().await;

    Err(Error::Generic)
}

async fn get_signer_pubkey(client: &Client) -> XOnlyPublicKey {
    client
        .subscribe(vec![Filter::new()
            .pubkey(client.keys().public_key())
            .kind(Kind::NostrConnect)
            .since(Timestamp::now())])
        .await;

    loop {
        let mut notifications = client.notifications();
        while let Ok(notification) = notifications.recv().await {
            if let RelayPoolNotification::Event(_url, event) = notification {
                if event.kind == Kind::NostrConnect {
                    match decrypt(
                        &client.keys().secret_key().unwrap(),
                        &event.pubkey,
                        &event.content,
                    ) {
                        Ok(msg) => {
                            let msg = Message::from_json(msg).unwrap();
                            if let Ok(Request::Connect(pubkey)) = msg.to_request() {
                                client.unsubscribe().await;
                                return pubkey;
                            }
                        }
                        Err(e) => eprintln!("Impossible to decrypt NIP46 message: {e}"),
                    }
                }
            }
        }
    }
}
