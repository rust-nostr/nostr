// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use nostr_sdk::prelude::*;

const BECH32_SK: &str = "nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85";

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let secret_key = SecretKey::from_bech32(BECH32_SK)?;
    let my_keys = Keys::new(secret_key);

    let proxy = Some(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 9050)));

    let client = Client::new(&my_keys);
    client.add_relay("ws://127.0.0.1:8080", None).await?;
    client.add_relay("wss://relay.nostr.info", proxy).await?;
    client.add_relay("wss://rsslay.fiatjaf.com", None).await?;
    client.add_relay("wss://relay.damus.io", None).await?;
    client.add_relay("wss://nostr.openchain.fr", None).await?;
    client
        .add_relay(
            "ws://jgqaglhautb4k6e6i2g34jakxiemqp6z4wynlirltuukgkft2xuglmqd.onion",
            proxy,
        )
        .await?;

    client.connect().await;

    client.send_event(Event::from_json(r#"{"id":"378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7","pubkey":"79dff8f82963424e0bb02708a22e44b4980893e3a4be0fa3cb60a43b946764e3","created_at":1671739153,"kind":4,"tags":[["p","68d81165918100b7da43fc28f7d1fc12554466e1115886b9e7bb326f65ec4272"]],"content":"8y4MRYrb4ztvXO2NmsHvUA==?iv=MplZo7oSdPfH/vdMC8Hmwg==","sig":"fd0954de564cae9923c2d8ee9ab2bf35bc19757f8e328a978958a2fcc950eaba0754148a203adec29b7b64080d0cf5a32bebedd768ea6eb421a6b751bb4584a8"}"#)?).await?;

    client
        .delete_event(
            EventId::from_hex("57689882a98ac4db67933196c121489dea7e1231f7c0f20accad4de838500edc")?,
            Some("reason"),
        )
        .await?;

    let entity: Entity = client
        .get_entity_of("25e5c82273a271cb1a840d0060391a0bf4965cafeb029d5ab55350b418953fbb")
        .await?;
    println!("Entity: {:?}", entity);

    let subscription = SubscriptionFilter::new()
        .pubkey(my_keys.public_key())
        .since(Timestamp::now());

    client.subscribe(vec![subscription]).await?;

    loop {
        let mut notifications = client.notifications();
        while let Ok(notification) = notifications.recv().await {
            if let RelayPoolNotification::Event(_url, event) = notification {
                if event.kind == Kind::EncryptedDirectMessage {
                    if let Ok(msg) = decrypt(&my_keys.secret_key()?, &event.pubkey, &event.content)
                    {
                        println!("New DM: {}", msg);
                    } else {
                        log::error!("Impossible to decrypt direct message");
                    }
                } else {
                    println!("{:?}", event);
                }
            }
        }
    }
}
