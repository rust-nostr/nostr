// Copyright (c) 2022 Thomas (0xtlt)
// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP11: Relay Information Document
//!
//! <https://github.com/nostr-protocol/nips/blob/master/11.md>

use alloc::string::String;
use alloc::vec::Vec;

use crate::{JsonUtil, Timestamp};

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

impl RelayInformationDocument {
    /// Create a new empty [`RelayInformationDocument`].
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }
}

impl JsonUtil for RelayInformationDocument {
    type Err = serde_json::Error;
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
        let expected = "[0,1,[5,7],[40,49]]";

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

        let doc = RelayInformationDocument::from_json(json).unwrap();

        assert_eq!(doc.name, Some(String::from("Test Relay")));
        assert_eq!(
            doc.description,
            Some(String::from("A test relay for unit testing"))
        );
    }

    #[test]
    fn serialization_round_trip() {
        let mut doc = RelayInformationDocument::new();
        doc.name = Some(String::from("Round Trip Test"));
        doc.supported_nips = Some(vec![1, 9, 11]);

        let json = doc.as_json();
        let parsed_doc = RelayInformationDocument::from_json(&json).unwrap();

        assert_eq!(doc, parsed_doc);
    }

    #[test]
    fn handles_invalid_json() {
        let invalid_json = r#"{"name": "Invalid", "supported_nips": [1, 2, "invalid"]}"#;
        let result = RelayInformationDocument::from_json(invalid_json);
        assert!(result.is_err());
    }
}
