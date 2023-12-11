// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use nostr_indexeddb::database::NostrDatabase;
use nostr_indexeddb::nostr::prelude::*;
use nostr_indexeddb::WebDatabase;
use wasm_bindgen_futures::spawn_local;
use web_sys::console;
use yew::prelude::*;

#[function_component(App)]
pub fn app() -> Html {
    spawn_local(async {
        let secret_key = SecretKey::from_bech32(
            "nsec1j4c6269y9w0q2er2xjw8sv2ehyrtfxq3jwgdlxj6qfn8z4gjsq5qfvfk99",
        )
        .unwrap();
        let keys_a = Keys::new(secret_key);
        console::log_1(&format!("Pubkey A: {}", keys_a.public_key()).into());

        let database = WebDatabase::open("nostr-sdk-indexeddb-test").await.unwrap();

        let metadata = Metadata::new().name("Name");
        let event = EventBuilder::set_metadata(&metadata)
            .to_event(&keys_a)
            .unwrap();
        database.save_event(&event).await.unwrap();

        let events = database
            .query(vec![Filter::new()
                .kinds(vec![Kind::Metadata, Kind::Custom(123), Kind::TextNote])
                .limit(20)
                .author(keys_a.public_key())])
            .await
            .unwrap();
        console::log_1(&format!("Events: {events:?}").into());
        console::log_1(&format!("Got {} events", events.len()).into());
    });

    html! {
        <main>
            <img class="logo" src="https://yew.rs/img/logo.png" alt="Yew logo" />
            <h1>{ "Hello World!" }</h1>
            <span class="subtitle">{ "from Yew with " }<i class="heart" /></span>
        </main>
    }
}
