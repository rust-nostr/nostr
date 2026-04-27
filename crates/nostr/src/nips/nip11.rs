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
    /// Link to relay's fee schedules
    pub payments_url: Option<String>,
    /// Relay fee schedules
    pub fees: Option<FeeSchedules>,
    /// URL pointing to an image to be used as an icon for the relay
    pub icon: Option<String>,
    /// Banner
    pub banner: Option<String>,
    /// Relay's own pubkey
    #[serde(rename = "self")]
    pub self_pubkey: Option<String>,
    /// Term of service
    pub terms_of_service: Option<String>,
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
    /// Relay will clamp each filter's limit value to this number
    pub max_limit: Option<i32>,
    /// Maximum length of subscription id as a string
    pub max_subid_length: Option<i32>,
    /// Maximum number of elements in the tags list
    pub max_event_tags: Option<i32>,
    /// Maximum number of characters in the content field of any event
    pub max_content_length: Option<i32>,
    /// New events will require at least this difficulty of PoW
    pub min_pow_difficulty: Option<i32>,
    /// Relay requires NIP42 authentication to happen before a new connection may perform any other action
    pub auth_required: Option<bool>,
    /// Relay requires payment before a new connection may perform any action
    pub payment_required: Option<bool>,
    /// Relay requires some kind of condition to be fulfilled to accept events
    pub restricted_writes: Option<bool>,
    /// 'created_at' lower limit
    pub created_at_lower_limit: Option<Timestamp>,
    /// 'created_at' upper limit
    pub created_at_upper_limit: Option<Timestamp>,
    /// Maximum returned events if you send a filter without a `limit`
    pub default_limit: Option<i32>,
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
    pub kinds: Option<Vec<u16>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correctly_parses_relay_information_document() {
        let json = r#"{
            "name": "Test Relay",
            "description": "A test relay for unit testing",
            "banner": "https://example.com/banner.webp",
            "icon": "https://example.com/icon.webp",
            "pubkey": "bf2bee5281149c7c350f5d12ae32f514c7864ff10805182f4178538c2c421007",
            "self": "79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798",
            "contact": "test@example.com",
            "supported_nips": [1, 9, 11],
            "software": "https://github.com/example/relay",
            "version": "1.0.0",
            "terms_of_service": "https://example.com/tos",
            "limitation": {
                "auth_required": false,
                "created_at_lower_limit": 94608000,
                "created_at_upper_limit": 300,
                "max_event_tags": 4000,
                "max_limit": 1000,
                "max_message_length": 16384,
                "max_subid_length": 71,
                "max_subscriptions": 300,
                "min_pow_difficulty": 0,
                "payment_required": true,
                "restricted_writes": true
            },
            "payments_url": "https://example.com",
            "fees": {
                "admission": [
                    {
                        "amount": 1000000,
                        "unit": "msats"
                    }
                ],
                "subscription": [
                    {
                        "amount": 5000000,
                        "unit": "msats",
                        "period": 2592000
                    }
                ],
                "publication": [
                    {
                        "kinds": [4],
                        "amount": 100,
                        "unit": "msats"
                    }
                ]
            }
        }"#;

        let doc = RelayInformationDocument::from_json(json).unwrap();

        assert_eq!(doc.name, Some(String::from("Test Relay")));
        assert_eq!(
            doc.self_pubkey,
            Some(String::from(
                "79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798"
            ))
        );
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
