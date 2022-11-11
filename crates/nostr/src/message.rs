// Copyright (c) 2021 Paul Miller
// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use chrono::{DateTime, Utc};
use secp256k1::XOnlyPublicKey;
use serde_json::{json, Value};
use thiserror::Error;
use uuid::Uuid;

use crate::{Event, Kind};

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct SubscriptionFilter {
    #[serde(skip_serializing_if = "Option::is_none")]
    ids: Option<Vec<Uuid>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    authors: Option<Vec<XOnlyPublicKey>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    kinds: Option<Vec<Kind>>,
    #[serde(rename = "#e")]
    #[serde(skip_serializing_if = "Option::is_none")]
    events: Option<Vec<Uuid>>,
    #[serde(rename = "#p")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pubkeys: Option<Vec<XOnlyPublicKey>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    since: Option<u64>, // unix timestamp seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    until: Option<u64>, // unix timestamp seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<u16>,
}

impl Default for SubscriptionFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl SubscriptionFilter {
    pub fn new() -> Self {
        Self {
            ids: None,
            kinds: None,
            events: None,
            pubkeys: None,
            since: None,
            until: None,
            authors: None,
            limit: None,
        }
    }

    /// Set subscription id
    pub fn id(self, id: impl Into<Uuid>) -> Self {
        Self {
            ids: Some(vec![id.into()]),
            ..self
        }
    }

    /// Set subscription ids
    pub fn ids(self, ids: impl Into<Vec<Uuid>>) -> Self {
        Self {
            ids: Some(ids.into()),
            ..self
        }
    }

    /// Set authors
    pub fn authors(self, authors: Vec<XOnlyPublicKey>) -> Self {
        Self {
            authors: Some(authors),
            ..self
        }
    }

    /// Set kind
    pub fn kind(self, kind: Kind) -> Self {
        Self {
            kinds: Some(vec![kind]),
            ..self
        }
    }

    /// Set kinds
    pub fn kinds(self, kinds: Vec<Kind>) -> Self {
        Self {
            kinds: Some(kinds),
            ..self
        }
    }

    /// Set events
    pub fn events(self, ids: impl Into<Vec<Uuid>>) -> Self {
        Self {
            events: Some(ids.into()),
            ..self
        }
    }

    /// Set pubkey
    pub fn pubkey(self, pubkey: XOnlyPublicKey) -> Self {
        Self {
            pubkeys: Some(vec![pubkey]),
            ..self
        }
    }

    /// Set pubkeys
    pub fn pubkeys(self, pubkeys: Vec<XOnlyPublicKey>) -> Self {
        Self {
            pubkeys: Some(pubkeys),
            ..self
        }
    }

    /// Set since
    pub fn since(self, since: DateTime<Utc>) -> Self {
        Self {
            since: Some(since.timestamp().try_into().unwrap_or(0)),
            ..self
        }
    }

    /// Set until
    pub fn until(self, until: DateTime<Utc>) -> Self {
        Self {
            until: Some(until.timestamp().try_into().unwrap_or(0)),
            ..self
        }
    }

    /// Set limit
    pub fn limit(self, limit: u16) -> Self {
        Self {
            limit: Some(limit),
            ..self
        }
    }
}

#[derive(Error, Debug, Eq, PartialEq)]
pub enum MessageHandleError {
    #[error("Message has an invalid format")]
    InvalidMessageFormat,
    #[error("Json deserialization failed")]
    JsonDeserializationFailed,
}

/// Messages sent by relays, received by clients
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum RelayMessage {
    //["EVENT", <subscription id>, <event JSON as defined above>]
    Event {
        event: Box<Event>,
        subscription_id: String,
    },
    Notice {
        message: String,
    },
    EndOfStoredEvents {
        subscription_id: String,
    },
    Empty,
}

impl RelayMessage {
    // Relay is responsible for storing corresponding subscription id
    pub fn new_event(event: Event, subscription_id: String) -> Self {
        Self::Event {
            event: Box::new(event),
            subscription_id,
        }
    }

    pub fn new_notice(message: String) -> Self {
        Self::Notice { message }
    }

    pub fn new_eose(subscription_id: String) -> Self {
        Self::EndOfStoredEvents { subscription_id }
    }

    pub fn to_json(&self) -> String {
        match self {
            Self::Empty => String::new(),
            Self::Event {
                event,
                subscription_id,
            } => json!(["EVENT", subscription_id, event]).to_string(),
            Self::Notice { message } => json!(["NOTICE", message]).to_string(),
            Self::EndOfStoredEvents { subscription_id } => {
                json!(["EOSE", subscription_id]).to_string()
            }
        }
    }

    pub fn from_json(msg: &str) -> Result<Self, MessageHandleError> {
        if msg.is_empty() {
            return Ok(Self::Empty);
        }

        let v: Vec<Value> =
            serde_json::from_str(msg).map_err(|_| MessageHandleError::JsonDeserializationFailed)?;

        // Notice
        // Relay response format: ["NOTICE", <message>]
        if v[0] == "NOTICE" {
            if v.len() != 2 {
                return Err(MessageHandleError::InvalidMessageFormat);
            }
            let v_notice: String = serde_json::from_value(v[1].clone())
                .map_err(|_| MessageHandleError::JsonDeserializationFailed)?;
            return Ok(Self::Notice { message: v_notice });
        }

        // Event
        // Relay response format: ["EVENT", <subscription id>, <event JSON>]
        if v[0] == "EVENT" {
            if v.len() != 3 {
                return Err(MessageHandleError::InvalidMessageFormat);
            }

            let event = Event::from_json(v[2].to_string())
                .map_err(|_| MessageHandleError::JsonDeserializationFailed)?;
            let subscription_id: String = serde_json::from_value(v[1].clone())
                .map_err(|_| MessageHandleError::JsonDeserializationFailed)?;

            return Ok(Self::new_event(event, subscription_id));
        }

        // EOSE (NIP-15)
        // Relay response format: ["EOSE", <subscription_id>]
        if v[0] == "EOSE" {
            if v.len() != 2 {
                return Err(MessageHandleError::InvalidMessageFormat);
            }

            let subscription_id: String = serde_json::from_value(v[1].clone())
                .map_err(|_| MessageHandleError::JsonDeserializationFailed)?;

            return Ok(Self::new_eose(subscription_id));
        }

        Err(MessageHandleError::InvalidMessageFormat)
    }
}

/// Messages sent by clients, received by relays
#[derive(Debug, Eq, PartialEq)]
pub enum ClientMessage {
    Event {
        event: Event,
    },
    Req {
        subscription_id: String,
        filters: Vec<SubscriptionFilter>,
    },
    Close {
        subscription_id: String,
    },
}

impl ClientMessage {
    pub fn new_event(event: Event) -> Self {
        Self::Event { event }
    }

    pub fn new_req(subscription_id: impl Into<String>, filters: Vec<SubscriptionFilter>) -> Self {
        Self::Req {
            subscription_id: subscription_id.into(),
            filters,
        }
    }

    pub fn close(subscription_id: String) -> Self {
        Self::Close { subscription_id }
    }

    pub fn to_json(&self) -> String {
        match self {
            Self::Event { event } => json!(["EVENT", event]).to_string(),
            Self::Req {
                subscription_id,
                filters,
            } => {
                let mut json = json!(["REQ", subscription_id]);
                let mut filters = json!(filters);

                if let Some(json) = json.as_array_mut() {
                    if let Some(filters) = filters.as_array_mut() {
                        json.append(filters);
                    }
                }

                json.to_string()
            }
            Self::Close { subscription_id } => json!(["CLOSE", subscription_id]).to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::{error::Error, str::FromStr};

    use uuid::uuid;

    use crate::KindBase;

    type TestResult = Result<(), Box<dyn Error>>;

    #[test]
    fn test_handle_valid_subscription_filter_multiple_id_prefixes() -> TestResult {
        let id_prefixes = vec![
            uuid!("b6527a19-5961-4310-8cf9-2d35307f442b"),
            uuid!("6b9cb378-2abd-439f-953b-883380e2701f"),
        ];
        let f = SubscriptionFilter::new().ids(id_prefixes.clone());

        assert_eq!(Some(id_prefixes), f.ids);

        Ok(())
    }

    #[test]
    fn test_handle_valid_notice() -> TestResult {
        let valid_notice_msg = r#"["NOTICE","Invalid event format!"]"#;
        let handled_valid_notice_msg =
            RelayMessage::new_notice(String::from("Invalid event format!"));

        assert_eq!(
            RelayMessage::from_json(valid_notice_msg)?,
            handled_valid_notice_msg
        );

        Ok(())
    }
    #[test]
    fn test_handle_invalid_notice() {
        //Missing content
        let invalid_notice_msg = r#"["NOTICE"]"#;
        //The content is not string
        let invalid_notice_msg_content = r#"["NOTICE": 404]"#;

        assert_eq!(
            RelayMessage::from_json(invalid_notice_msg).unwrap_err(),
            MessageHandleError::InvalidMessageFormat
        );
        assert_eq!(
            RelayMessage::from_json(invalid_notice_msg_content).unwrap_err(),
            MessageHandleError::JsonDeserializationFailed
        );
    }

    #[test]
    fn test_handle_valid_event() -> TestResult {
        let valid_event_msg = r#"["EVENT", "random_string", {"id":"70b10f70c1318967eddf12527799411b1a9780ad9c43858f5e5fcd45486a13a5","pubkey":"379e863e8357163b5bce5d2688dc4f1dcc2d505222fb8d74db600f30535dfdfe","created_at":1612809991,"kind":1,"tags":[],"content":"test","sig":"273a9cd5d11455590f4359500bccb7a89428262b96b3ea87a756b770964472f8c3e87f5d5e64d8d2e859a71462a3f477b554565c4f2f326cb01dd7620db71502"}]"#;

        let id = "70b10f70c1318967eddf12527799411b1a9780ad9c43858f5e5fcd45486a13a5";
        let pubkey = "379e863e8357163b5bce5d2688dc4f1dcc2d505222fb8d74db600f30535dfdfe";
        let created_at = 1612809991;
        let kind = 1;
        let tags = vec![];
        let content = "test";
        let sig = "273a9cd5d11455590f4359500bccb7a89428262b96b3ea87a756b770964472f8c3e87f5d5e64d8d2e859a71462a3f477b554565c4f2f326cb01dd7620db71502";

        let handled_event = Event::new_dummy(id, pubkey, created_at, kind, tags, content, sig);

        assert_eq!(
            RelayMessage::from_json(valid_event_msg)?,
            RelayMessage::new_event(handled_event?, "random_string".to_string())
        );

        Ok(())
    }

    #[test]
    fn test_handle_invalid_event() {
        //Mising Event field
        let invalid_event_msg = r#"["EVENT", "random_string"]"#;
        //Event JSON with incomplete content
        let invalid_event_msg_content = r#"["EVENT", "random_string", {"id":"70b10f70c1318967eddf12527799411b1a9780ad9c43858f5e5fcd45486a13a5","pubkey":"379e863e8357163b5bce5d2688dc4f1dcc2d505222fb8d74db600f30535dfdfe"}]"#;

        assert_eq!(
            RelayMessage::from_json(invalid_event_msg).unwrap_err(),
            MessageHandleError::InvalidMessageFormat
        );

        assert_eq!(
            RelayMessage::from_json(invalid_event_msg_content).unwrap_err(),
            MessageHandleError::JsonDeserializationFailed
        );
    }

    #[test]
    fn test_handle_valid_eose() -> TestResult {
        let valid_eose_msg = r#"["EOSE","random-subscription-id"]"#;
        let handled_valid_eose_msg = RelayMessage::new_eose(String::from("random-subscription-id"));

        assert_eq!(
            RelayMessage::from_json(valid_eose_msg)?,
            handled_valid_eose_msg
        );

        Ok(())
    }
    #[test]
    fn test_handle_invalid_eose() {
        // Missing subscription ID
        assert_eq!(
            RelayMessage::from_json(r#"["EOSE"]"#).unwrap_err(),
            MessageHandleError::InvalidMessageFormat
        );

        // The subscription ID is not string
        assert_eq!(
            RelayMessage::from_json(r#"["EOSE", 404]"#).unwrap_err(),
            MessageHandleError::JsonDeserializationFailed
        );
    }

    #[test]
    fn test_client_message_req() {
        let pk = XOnlyPublicKey::from_str(
            "379e863e8357163b5bce5d2688dc4f1dcc2d505222fb8d74db600f30535dfdfe",
        )
        .unwrap();
        let filters = vec![
            SubscriptionFilter::new().kind(Kind::Base(KindBase::EncryptedDirectMessage)),
            SubscriptionFilter::new().pubkey(pk),
        ];

        let client_req = ClientMessage::new_req("test", filters);
        assert_eq!(
            client_req.to_json(),
            r##"["REQ","test",{"kinds":[4]},{"#p":["379e863e8357163b5bce5d2688dc4f1dcc2d505222fb8d74db600f30535dfdfe"]}]"##
        );
    }

    #[test]
    fn test_client_message_custom_kind() {
        let pk = XOnlyPublicKey::from_str(
            "379e863e8357163b5bce5d2688dc4f1dcc2d505222fb8d74db600f30535dfdfe",
        )
        .unwrap();
        let filters = vec![
            SubscriptionFilter::new().kind(Kind::Custom(22)),
            SubscriptionFilter::new().pubkey(pk),
        ];

        let client_req = ClientMessage::new_req("test", filters);
        assert_eq!(
            client_req.to_json(),
            r##"["REQ","test",{"kinds":[22]},{"#p":["379e863e8357163b5bce5d2688dc4f1dcc2d505222fb8d74db600f30535dfdfe"]}]"##
        );
    }
}
