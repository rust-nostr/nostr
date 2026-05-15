// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::time::Duration;

use nostr::event::EventBuilder;
use nostr::key::{Keys, PublicKey};
use nostr::nips::nip46::{NostrConnectRequest, NostrConnectUri};
use nostr::types::RelayUrl;
use nostr_connect::client::NostrConnect;
use nostr_connect::signer::{
    NostrConnectKeys, NostrConnectRemoteSigner, NostrConnectSignerActions,
};
use nostr_relay_builder::LocalRelayBuilder;

struct MySignerActions;

impl NostrConnectSignerActions for MySignerActions {
    fn approve(&self, _public_key: &PublicKey, _req: &NostrConnectRequest) -> bool {
        true
    }
}

async fn test_bunker_uri(user_keys: Keys, relay_url: RelayUrl) {
    let bunker_keys = Keys::generate();
    let secret = Some("urmom".to_owned());
    let bunker = NostrConnectRemoteSigner::new(
        NostrConnectKeys {
            signer: bunker_keys,
            user: user_keys.clone(),
        },
        [relay_url.clone()],
        secret,
        None,
    )
    .unwrap();

    let bunker_uri = bunker.bunker_uri();
    tokio::spawn(async move {
        bunker.serve(MySignerActions).await.unwrap();
    });

    // Make sure the bunker started
    tokio::time::sleep(Duration::from_millis(500)).await;

    let nostr_connect_signer = NostrConnect::new(
        NostrConnectUri::parse(bunker_uri.to_string()).unwrap(),
        Keys::generate(),
        Duration::from_secs(5),
        None,
    )
    .unwrap();

    let event = EventBuilder::text_note("GM")
        .sign_async(&nostr_connect_signer)
        .await
        .unwrap();

    assert_eq!(event.pubkey, user_keys.public_key);
    assert!(event.verify().is_ok());
}

async fn test_bunker_uri_no_secret(user_keys: Keys, relay_url: RelayUrl) {
    let bunker_keys = Keys::generate();
    let bunker = NostrConnectRemoteSigner::new(
        NostrConnectKeys {
            signer: bunker_keys,
            user: user_keys.clone(),
        },
        [relay_url.clone()],
        None,
        None,
    )
    .unwrap();

    let bunker_uri = bunker.bunker_uri();
    tokio::spawn(async move {
        bunker.serve(MySignerActions).await.unwrap();
    });

    // Make sure the bunker started
    tokio::time::sleep(Duration::from_millis(500)).await;

    let nostr_connect_signer = NostrConnect::new(
        NostrConnectUri::parse(bunker_uri.to_string()).unwrap(),
        Keys::generate(),
        Duration::from_secs(5),
        None,
    )
    .unwrap();

    let event = EventBuilder::text_note("GM")
        .sign_async(&nostr_connect_signer)
        .await
        .unwrap();

    assert_eq!(event.pubkey, user_keys.public_key);
    assert!(event.verify().is_ok());
}

async fn test_nostrconnect_uri(relay_url: RelayUrl, user_keys: Keys) {
    let app_keys = Keys::generate();
    let connect_uri = NostrConnectUri::client(app_keys.public_key, [relay_url], "Test App");

    let bunker = NostrConnectRemoteSigner::from_uri(
        connect_uri.clone(),
        NostrConnectKeys {
            signer: Keys::generate(),
            user: user_keys.clone(),
        },
        None,
    )
    .unwrap();

    tokio::spawn(async move {
        // Wait for the client to start listening. In the `nostrconnect` flow,
        // the client begins listening before the bunker is ready.
        tokio::time::sleep(Duration::from_millis(500)).await;
        bunker.serve(MySignerActions).await.unwrap();
    });

    let nostr_connect_signer =
        NostrConnect::new(connect_uri, app_keys, Duration::from_secs(5), None).unwrap();

    let event = EventBuilder::text_note("GM")
        .sign_async(&nostr_connect_signer)
        .await
        .unwrap();

    assert_eq!(event.pubkey, user_keys.public_key);
    assert!(event.verify().is_ok());
}

#[tokio::test]
async fn nostr_connect_bunker_uri() {
    let relay = LocalRelayBuilder::default().build();
    let relay_url = relay.url().await;
    relay.run().await.unwrap();

    let user_keys = Keys::generate();

    test_bunker_uri(user_keys.clone(), relay_url.clone()).await;
}

#[tokio::test]
async fn nostr_connect_bunker_uri_no_secret() {
    let relay = LocalRelayBuilder::default().build();
    let relay_url = relay.url().await;
    relay.run().await.unwrap();

    let user_keys = Keys::generate();

    test_bunker_uri_no_secret(user_keys.clone(), relay_url.clone()).await;
}

#[tokio::test]
async fn nostr_connect_nostrconnect_uri() {
    let relay = LocalRelayBuilder::default().build();
    let relay_url = relay.url().await;
    relay.run().await.unwrap();

    let user_keys = Keys::generate();

    test_nostrconnect_uri(relay_url, user_keys).await;
}
