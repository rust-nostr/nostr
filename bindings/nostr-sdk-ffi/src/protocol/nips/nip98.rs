// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::str::FromStr;

use nostr::hashes::sha256::Hash as Sha256Hash;
use nostr::nips::nip98;
use nostr::Url;
use uniffi::{Enum, Record};

use crate::error::NostrSdkError;

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

impl TryFrom<HttpData> for nip98::HttpData {
    type Error = NostrSdkError;

    fn try_from(value: HttpData) -> Result<Self, Self::Error> {
        Ok(Self {
            url: Url::parse(&value.url)?,
            method: value.method.into(),
            payload: match value.payload {
                Some(p) => Sha256Hash::from_str(&p).ok(),
                None => None,
            },
        })
    }
}
