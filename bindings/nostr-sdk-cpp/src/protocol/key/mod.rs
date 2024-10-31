// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use anyhow::Result;
use cxx::CxxString;
use nostr_sdk::nostr::key;

#[cxx::bridge(namespace = "key")]
mod ffi {
    extern "Rust" {
        type Keys;

        fn generate() -> Box<Keys>;

        fn parse(secret_key: &CxxString) -> Result<Box<Keys>>;

        fn public_key(&self) -> String;
    }
}

pub struct Keys {
    inner: key::Keys,
}

pub fn generate() -> Box<Keys> {
    Box::new(Keys {
        inner: key::Keys::generate(),
    })
}

pub fn parse(secret_key: &CxxString) -> Result<Box<Keys>> {
    let secret_key: &str = secret_key.to_str()?;
    Ok(Box::new(Keys {
        inner: key::Keys::parse(secret_key)?,
    }))
}

impl Keys {
    pub fn public_key(&self) -> String {
        self.inner.public_key.to_string()
    }
}
