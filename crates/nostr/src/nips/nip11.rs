// Copyright (c) 2022 Thomas (0xtlt)
// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP11: Relay Information Document
//!
//! <https://github.com/nostr-protocol/nips/blob/master/11.md>

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;
use std::net::SocketAddr;

use reqwest::Client;
#[cfg(not(target_arch = "wasm32"))]
use reqwest::Proxy;

use crate::{Timestamp, Url};

/// `NIP11` error
#[derive(Debug)]
pub enum Error {
    /// The relay information document is invalid
    InvalidInformationDocument,
    /// The relay information document is not accessible
    InaccessibleInformationDocument,
    /// Provided URL scheme is not valid
    InvalidScheme,
    /// Reqwest error
    Reqwest(reqwest::Error),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInformationDocument => {
                write!(f, "The relay information document is invalid")
            }
            Self::InaccessibleInformationDocument => {
                write!(f, "The relay information document is not accessible")
            }
            Self::InvalidScheme => write!(f, "Provided URL scheme is not valid"),
            Self::Reqwest(e) => write!(f, "{e}"),
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Self::Reqwest(e)
    }
}

/// Relay information document
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct RelayInformationDocument {
    /// Name
    pub name: Option<String>,
    /// Description
    pub description: Option<String>,
    /// Owner public key
    pub pubkey: Option<String>,
    /// Owner contact
    pub contact: Option<String>,
    /// Supported NIPs
    pub supported_nips: Option<Vec<u16>>,
    /// Software
    pub software: Option<String>,
    /// Software version
    pub version: Option<String>,
    /// Limitations imposed by the relay on clients
    pub limitation: Option<Limitation>,
    /// The relay's retention policies
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub retention: Vec<Retention>,
    /// Country codes whose laws and policies may affect this relay
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub relay_countries: Vec<String>,
    /// Ordered list of IETF language tags indicating the major languages spoken on the relay
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub language_tags: Vec<String>,
    /// List of limitations on the topics to be discussed
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub tags: Vec<String>,
    /// Link to a human-readable page which specifies the community policies
    pub posting_policy: Option<String>,
    /// Link to relay's fee schedules
    pub payments_url: Option<String>,
    /// Relay fee schedules
    pub fees: Option<FeeSchedules>,
    /// URL pointing to an image to be used as an icon for the relay
    pub icon: Option<String>,
}

/// These are limitations imposed by the relay on clients. Your client should
/// expect that requests which exceed these practical limitations are rejected or fail immediately.
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Limitation {
    /// Maximum number of bytes for incoming JSON that the relay will attempt to decode and act upon
    pub max_message_length: Option<i32>,
    /// Total number of subscriptions that may be active on a single websocket connection
    pub max_subscriptions: Option<i32>,
    /// Maximum number of filter values in each subscription
    pub max_filters: Option<i32>,
    /// Relay will clamp each filter's limit value to this number
    pub max_limit: Option<i32>,
    /// Maximum length of subscription id as a string
    pub max_subid_length: Option<i32>,
    /// Maximum number of elements in the tags list
    pub max_event_tags: Option<i32>,
    /// Maximum number of characters in the content field of any event
    pub max_content_length: Option<i32>,
    /// New events will require at least this difficulty of PoW,
    pub min_pow_difficulty: Option<i32>,
    /// Relay requires NIP42 authentication to happen before a new connection may perform any other action
    pub auth_required: Option<bool>,
    /// Relay requires payment before a new connection may perform any action
    pub payment_required: Option<bool>,
    /// 'created_at' lower limit
    pub created_at_lower_limit: Option<Timestamp>,
    /// 'created_at' upper limit
    pub created_at_upper_limit: Option<Timestamp>,
}

/// A retention schedule for the relay
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Retention {
    /// The event kinds this retention pertains to
    pub kinds: Option<Vec<RetentionKind>>,
    /// The amount of time these events are kept
    pub time: Option<u64>,
    /// The max number of events kept before removing older events
    pub count: Option<u64>,
}

/// A single kind or range of kinds the retention pertains to
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RetentionKind {
    /// A single kind
    Single(u64),
    /// A kind range
    Range(u64, u64),
}

/// Available fee schedules
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct FeeSchedules {
    /// Fees for admission to use the relay
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub admission: Vec<FeeSchedule>,
    /// Fees for subscription to use the relay
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub subscription: Vec<FeeSchedule>,
    /// Fees to publish to the relay
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub publication: Vec<FeeSchedule>,
}

/// The specific information about a fee schedule
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct FeeSchedule {
    /// The fee amount
    pub amount: i32,
    /// The denomination of the feed
    pub unit: String,
    /// The duration for which the fee is valid
    pub period: Option<i32>,
    /// The event kinds the fee allows the client to publish to the relay
    pub kinds: Option<Vec<String>>,
}

impl RelayInformationDocument {
    /// Create new empty [`RelayInformationDocument`]
    pub fn new() -> Self {
        Self::default()
    }

    /// Get Relay Information Document
    ///
    /// **Proxy is ignored for WASM targets!**
    pub async fn get(mut url: Url, _proxy: Option<SocketAddr>) -> Result<Self, Error> {
        #[cfg(not(target_arch = "wasm32"))]
        let client: Client = {
            let mut builder = Client::builder();
            if let Some(proxy) = _proxy {
                let proxy = format!("socks5h://{proxy}");
                builder = builder.proxy(Proxy::all(proxy)?);
            }
            builder.build()?
        };

        #[cfg(target_arch = "wasm32")]
        let client: Client = Client::new();

        let url: &str = Self::with_http_scheme(&mut url)?;
        let req = client.get(url).header("Accept", "application/nostr+json");
        match req.send().await {
            Ok(response) => {
                let json: String = response.text().await?;
                match serde_json::from_slice(json.as_bytes()) {
                    Ok(json) => Ok(json),
                    Err(_) => Err(Error::InvalidInformationDocument),
                }
            }
            Err(_) => Err(Error::InaccessibleInformationDocument),
        }
    }

    /// Returns new URL with scheme substituted to HTTP(S) if WS(S) was provided,
    /// other schemes leaves untouched.
    fn with_http_scheme(url: &mut Url) -> Result<&str, Error> {
        match url.scheme() {
            "wss" => url.set_scheme("https").map_err(|_| Error::InvalidScheme)?,
            "ws" => url.set_scheme("http").map_err(|_| Error::InvalidScheme)?,
            _ => {}
        }
        Ok(url.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correctly_serializes_retention_kind() {
        let kinds = vec![
            RetentionKind::Single(0),
            RetentionKind::Single(1),
            RetentionKind::Range(5, 7),
            RetentionKind::Range(40, 49),
        ];
        let got = serde_json::to_string(&kinds).unwrap();
        let expected = "[0,1,[5,7],[40,49]]".to_string();

        assert_eq!(got, expected, "got: {}, expected: {}", got, expected);
    }

    #[test]
    fn correctly_deserializes_retention_kind() {
        let kinds = "[0, 1, [5, 7], [40, 49]]";
        let got = serde_json::from_str::<Vec<RetentionKind>>(kinds).unwrap();
        let expected = vec![
            RetentionKind::Single(0),
            RetentionKind::Single(1),
            RetentionKind::Range(5, 7),
            RetentionKind::Range(40, 49),
        ];

        assert_eq!(got, expected, "got: {:?}, expected: {:?}", got, expected);
    }
}
