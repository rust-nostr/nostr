// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use anyhow::Result;
use cxx::CxxString;
use nostr_sdk::client;
use nostr_sdk::nostr::{Event, JsonUtil};

use crate::RUNTIME;

#[cxx::bridge(namespace = "client")]
mod ffi {
    extern "Rust" {
        type Client;

        fn without_signer() -> Box<Client>;

        fn add_relay(&self, url: &CxxString) -> Result<bool>;

        fn connect(&self);

        fn send_event(&self, json: &CxxString) -> Result<()>;
    }
}

pub struct Client {
    inner: client::Client,
}

fn without_signer() -> Box<Client> {
    Box::new(Client {
        inner: client::Client::default(),
    })
}

impl Client {
    pub fn add_relay(&self, url: &CxxString) -> Result<bool> {
        RUNTIME.block_on(async {
            let url: &str = url.to_str()?;
            Ok(self.inner.add_relay(url).await?)
        })
    }

    pub fn connect(&self) {
        RUNTIME.block_on(async { self.inner.connect().await })
    }

    pub fn send_event(&self, json: &CxxString) -> Result<()> {
        RUNTIME.block_on(async {
            let json: &str = json.to_str()?;
            let event = Event::from_json(json)?;
            self.inner.send_event(event).await?;
            Ok(())
        })
    }
}
