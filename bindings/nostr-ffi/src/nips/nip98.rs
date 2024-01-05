// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::str::FromStr;

use nostr::hashes::sha256::Hash as Sha256Hash;
use nostr::nips::nip98;
use nostr::UncheckedUrl;
use uniffi::Record;

use crate::event::tag::HttpMethod;

#[derive(Record)]
pub struct HttpData {
    pub url: String,
    pub method: HttpMethod,
    pub payload: Option<String>,
}

impl From<HttpData> for nip98::HttpData {
    fn from(value: HttpData) -> Self {
        Self {
            url: UncheckedUrl::from(value.url),
            method: value.method.into(),
            payload: match value.payload {
                Some(p) => Sha256Hash::from_str(&p).ok(),
                None => None,
            },
        }
    }
}
