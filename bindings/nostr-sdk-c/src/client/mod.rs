// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::ffi::{c_char, CStr};

use nostr_sdk::client;
use nostr_sdk::nostr::{Event, JsonUtil};

use crate::RUNTIME;

pub struct Client {
    inner: client::Client,
}

#[no_mangle]
pub extern "C" fn client_without_signer() -> *const Client {
    Box::into_raw(Box::new(Client {
        inner: client::Client::default(),
    }))
}

#[no_mangle]
pub unsafe extern "C" fn client_add_relay(client: &Client, url: *const c_char) -> bool {
    RUNTIME.block_on(async {
        let url: &str = CStr::from_ptr(url).to_str().unwrap();
        client.inner.add_relay(url).await.unwrap()
    })
}

#[no_mangle]
pub unsafe extern "C" fn client_connect(client: &Client) {
    RUNTIME.block_on(async { client.inner.connect().await })
}

#[no_mangle]
pub unsafe extern "C" fn client_send_event(client: &Client, json: *const c_char) {
    RUNTIME.block_on(async {
        let json: &str = CStr::from_ptr(json).to_str().unwrap();
        let event = Event::from_json(json).unwrap();
        client.inner.send_event(event).await.unwrap();
    })
}
