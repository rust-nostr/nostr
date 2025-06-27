// Copyright (c) 2022 Thomas (0xtlt)
// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP11: Relay Information Document
//!
//! <https://github.com/nostr-protocol/nips/blob/master/11.md>

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use crate::{Timestamp, Url};

/// `NIP11` error
#[derive(Debug)]
pub enum Error {
    /// The relay information document is invalid
    InvalidInformationDocument,
    /// Provided URL scheme is not valid
    InvalidScheme,
    /// JSON parsing error
    JsonParseError(serde_json::Error),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInformationDocument => {
                write!(f, "The relay information document is invalid")
            }
            Self::InvalidScheme => write!(f, "Provided URL scheme is not valid"),
            Self::JsonParseError(e) => write!(f, "JSON parsing error: {e}"),
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::JsonParseError(e)
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
    /// Relay requires some kind of condition to be fulfilled to accept events
    pub restricted_writes: Option<bool>,
    /// Maximum returned events if you send a filter with the limit set to 0
    pub default_limit: Option<i32>,
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
    /// Create a new empty [`RelayInformationDocument`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Parse amethod to parse JSON string into a [`RelayInformationDocument`]
    /// This method replaces the previous `get` method, allowing users to fetch
    /// the JSON data using their preferred HTTP client and then parse it here.
    pub fn parse(json: &str) -> Result<Self, Error> {
        serde_json::from_str(json).map_err(Error::from)
    }

    /// Parse method for bytes to handle different input formats
    /// Parse JSON bytes into a [`RelayInformationDocument`]
    /// Useful when working with HTTP responses that return bytes directly.
    pub fn parse_bytes(json: &[u8]) -> Result<Self, Error> {
        serde_json::from_slice(json).map_err(Error::from)
    }

    /// Serialize method for converting back to JSON
    /// Serializes the [`RelayInformationDocument`] to a JSON string
    pub fn to_json(&self) -> Result<String, Error> {
        serde_json::to_string(self).map_err(Error::from)
    }

    /// Pretty serialize method for human-readable JSON
    /// Serializes the [`RelayInformationDocument`] to a pretty-printed JSON string
    pub fn to_json_pretty(&self) -> Result<String, Error> {
        serde_json::to_string_pretty(self).map_err(Error::from)
    }

    /// Returns new URL with scheme substituted to HTTP(S) if WS(S) was provided,
    /// other schemes leaves untouched.
    pub fn with_http_scheme(url: &mut Url) -> Result<&str, Error> {
        match url.scheme() {
            "wss" => url.set_scheme("https").map_err(|_| Error::InvalidScheme)?,
            "ws" => url.set_scheme("http").map_err(|_| Error::InvalidScheme)?,
            _ => {}
        }
        Ok(url.as_str())
    }

    // A utility method that gets HTTP URL from a WebSocket URL without mutation
    /// Convert a WebSocket URL to HTTP URL for fetching the relay information
    /// Returns a new URL string with the scheme converted from ws/wss to http/https.
    /// Other schemes are returned as-is.
    pub fn get_http_url_from_ws(url: &Url) -> Result<String, Error> {
        let mut url_copy = url.clone();
        Self::with_http_scheme(&mut url_copy).map(|s| s.to_string())
    }

    /// This validation methods helps verify the document structure
    /// and check if the relay supports a specific NIP
    pub fn supports_nip(&self, nip: u16) -> bool {
        self.supported_nips
            .as_ref()
            .map(|nips| nips.contains(&nip))
            .unwrap_or(false)
    }

    /// Check if the relay needs authentication
    pub fn requires_auth(&self) -> bool {
        self.limitation
            .as_ref()
            .and_then(|l| l.auth_required)
            .unwrap_or(false)
    }

    /// Check if the relay needs payment
    pub fn requires_payment(&self) -> bool {
        self.limitation
            .as_ref()
            .and_then(|l| l.payment_required)
            .unwrap_or(false)
    }

    /// Check if the relay has restricted writes
    pub fn has_restricted_writes(&self) -> bool {
        self.limitation
            .as_ref()
            .and_then(|l| l.restricted_writes)
            .unwrap_or(false)
    }

    /// Get the maximum message length allowed by the relay
    pub fn max_message_length(&self) -> Option<i32> {
        self.limitation.as_ref().and_then(|l| l.max_message_length)
    }

    /// Get the maximum number of subscriptions allowed
    pub fn max_subscriptions(&self) -> Option<i32> {
        self.limitation.as_ref().and_then(|l| l.max_subscriptions)
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
    #[test]
    fn correctly_parses_relay_information_document() {
        let json = r#"{
            "name": "Test Relay",
            "description": "A test relay for unit testing",
            "pubkey": "bf2bee5281149c7c350f5d12ae32f514c7864ff10805182f4178538c2c421007",
            "contact": "test@example.com",
            "supported_nips": [1, 9, 11],
            "software": "https://github.com/example/relay",
            "version": "1.0.0",
            "limitation": {
                "max_message_length": 16384,
                "max_subscriptions": 300,
                "auth_required": false,
                "payment_required": true
            }
        }"#;

        let doc = RelayInformationDocument::parse(json).unwrap();

        assert_eq!(doc.name, Some("Test Relay".to_string()));
        assert_eq!(
            doc.description,
            Some("A test relay for unit testing".to_string())
        );
        assert!(doc.supports_nip(1));
        assert!(doc.supports_nip(9));
        assert!(doc.supports_nip(11));
        assert!(!doc.supports_nip(42));
        assert!(!doc.requires_auth());
        assert!(doc.requires_payment());
        assert_eq!(doc.max_message_length(), Some(16384));
        assert_eq!(doc.max_subscriptions(), Some(300));
    }

    #[test]
    fn correctly_converts_websocket_to_http_url() {
        let ws_url = Url::parse("ws://example.com/relay").unwrap();
        let http_url = RelayInformationDocument::get_http_url_from_ws(&ws_url).unwrap();
        assert_eq!(http_url, "http://example.com/relay");

        let wss_url = Url::parse("wss://example.com/relay").unwrap();
        let https_url = RelayInformationDocument::get_http_url_from_ws(&wss_url).unwrap();
        assert_eq!(https_url, "https://example.com/relay");

        let http_url = Url::parse("http://example.com/relay").unwrap();
        let unchanged_url = RelayInformationDocument::get_http_url_from_ws(&http_url).unwrap();
        assert_eq!(unchanged_url, "http://example.com/relay");
    }

    #[test]
    fn serialization_round_trip() {
        let mut doc = RelayInformationDocument::new();
        doc.name = Some("Round Trip Test".to_string());
        doc.supported_nips = Some(vec![1, 9, 11]);

        let json = doc.to_json().unwrap();
        let parsed_doc = RelayInformationDocument::parse(&json).unwrap();

        assert_eq!(doc, parsed_doc);
    }

    #[test]
    fn handles_invalid_json() {
        let invalid_json = r#"{"name": "Invalid", "supported_nips": [1, 2, "invalid"]}"#;
        let result = RelayInformationDocument::parse(invalid_json);

        assert!(result.is_err());
        match result.unwrap_err() {
            Error::JsonParseError(_) => {} // Expected
            _ => panic!("Expected JsonParseError"),
        }
    }
}
