// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::str::FromStr;

use nostr::hashes::sha256::Hash as Sha256Hash;
use nostr::nips::nip98;
use nostr::UncheckedUrl;
use uniffi::{Enum, Record};

#[derive(Enum)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
}

impl From<HttpMethod> for nip98::HttpMethod {
    fn from(value: HttpMethod) -> Self {
        match value {
            HttpMethod::Get => Self::GET,
            HttpMethod::Post => Self::POST,
            HttpMethod::Put => Self::PUT,
            HttpMethod::Patch => Self::PATCH,
        }
    }
}

impl From<nip98::HttpMethod> for HttpMethod {
    fn from(value: nip98::HttpMethod) -> Self {
        match value {
            nip98::HttpMethod::GET => Self::Get,
            nip98::HttpMethod::POST => Self::Post,
            nip98::HttpMethod::PUT => Self::Put,
            nip98::HttpMethod::PATCH => Self::Patch,
        }
    }
}

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
