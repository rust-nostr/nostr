use crate::{Event, Kind};
use chrono::{DateTime, Utc};
use secp256k1::XOnlyPublicKey;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use thiserror::Error;
use crate::event::KindBase;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct SubscriptionFilter {
    // TODO can we write this without all these "Option::is_none"
    #[serde(skip_serializing_if = "Option::is_none")]
    ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    kinds: Option<Vec<Kind>>,
    #[serde(rename = "#e")]
    #[serde(skip_serializing_if = "Option::is_none")]
    events: Option<Vec<String>>,
    #[serde(rename = "#p")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pubkeys: Option<Vec<XOnlyPublicKey>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    since: Option<u64>, // unix timestamp seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    until: Option<u64>, // unix timestamp seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    authors: Option<Vec<XOnlyPublicKey>>,
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
        }
    }

    pub fn id(self, id: impl Into<String>) -> Self {
        Self {
            ids: Some(vec![id.into()]),
            ..self
        }
    }

    pub fn ids(self, ids: impl Into<Vec<String>>) -> Self {
        Self {
            ids: Some(ids.into()),
            ..self
        }
    }

    pub fn kind_custom(self, kind_id: u16) -> Self {
        Self {
            kinds: Some(vec![Kind::Custom(kind_id)]),
            ..self
        }
    }

    pub fn kind_base(self, kind_base: KindBase) -> Self {
        Self {
            kinds: Some(vec![Kind::Base(kind_base)]),
            ..self
        }
    }

    // #e
    pub fn event(self, event_id: impl Into<String>) -> Self {
        Self {
            events: Some(vec![event_id.into()]),
            ..self
        }
    }

    // #p, for instance the receiver public key
    pub fn pubkey(self, pubkey: XOnlyPublicKey) -> Self {
        Self {
            pubkeys: Some(vec![pubkey]),
            ..self
        }
    }

    // unix timestamp seconds
    pub fn since(self, since: DateTime<Utc>) -> Self {
        Self {
            // TODO is there a cleaner way to do this
            since: Some(since.timestamp().try_into().unwrap_or(0)),
            ..self
        }
    }

    pub fn until(self, until: DateTime<Utc>) -> Self {
        Self {
            until: Some(until.timestamp().try_into().unwrap_or(0)),
            ..self
        }
    }

    pub fn authors(self, authors: Vec<XOnlyPublicKey>) -> Self {
        Self {
            authors: Some(authors),
            ..self
        }
    }
}

#[derive(Error, Debug, PartialEq)]
pub enum MessageHandleError {
    #[error("Message has an invalid format")]
    InvalidMessageFormat,

    #[error("Json deserialization failed")]
    JsonDeserializationFailed,
}

/// Messages sent by relays, received by clients
#[derive(Debug, PartialEq)]
pub enum RelayMessage {
    //["EVENT", <subscription id>, <event JSON as defined above>]
    Event {
        event: Event,
        subscription_id: String,
    },
    Notice {
        message: String,
    },
    EndOfStoredEvents {
        subscription_id: String,
    },
    // TODO: maybe we can remove this idk
    Empty,
}

impl RelayMessage {
    // Relay is responsible for storing corresponding subscription id
    pub fn new_event(event: Event, subscription_id: String) -> Self {
        Self::Event {
            event,
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
            Self::EndOfStoredEvents {subscription_id} => json!(["EOSE", subscription_id]).to_string(),
        }
    }

    pub fn from_json(msg: &str) -> Result<Self, MessageHandleError> {
        if msg == "" {
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

            let event = Event::new_from_json(v[2].to_string())
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

            return Ok(Self::new_eose( subscription_id ));
        }

        Err(MessageHandleError::InvalidMessageFormat)
    }
}

/// Messages sent by clients, received by relays
#[derive(Debug, PartialEq)]
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
                json.as_array_mut()
                    .unwrap()
                    .append(filters.as_array_mut().unwrap());
                return json.to_string();
            }
            Self::Close { subscription_id } => json!(["CLOSE", subscription_id]).to_string(),
        }
    }

    pub fn from_json(msg: &str) -> Result<Self, MessageHandleError> {
        dbg!(msg);

        let _v: Vec<Value> =
            serde_json::from_str(msg).map_err(|_| MessageHandleError::JsonDeserializationFailed)?;

        // Notice
        // Relay response format: ["NOTICE", <message>]
        // if v[0] == "NOTICE" {
        //     if v.len() != 2 {
        //         return Err(MessageHandleError::InvalidMessageFormat);
        //     }
        //     let v_notice: String = serde_json::from_value(v[1].clone())
        //         .map_err(|_| MessageHandleError::JsonDeserializationFailed)?;
        //     return Ok(Self::Notice { message: v_notice });
        // }

        // // Event
        // // Relay response format: ["EVENT", <subscription id>, <event JSON>]
        // if v[0] == "EVENT" {
        //     if v.len() != 3 {
        //         return Err(MessageHandleError::InvalidMessageFormat);
        //     }

        //     let event = Event::new_from_json(v[2].to_string())
        //         .map_err(|_| MessageHandleError::JsonDeserializationFailed)?;
        //     let _context = v[1].clone();

        //     return Ok(Self::Event { event });
        // }

        Err(MessageHandleError::InvalidMessageFormat)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::{error::Error, str::FromStr};

    type TestResult = Result<(), Box<dyn Error>>;

    #[test]
    fn test_handle_valid_subscription_filter_multiple_id_prefixes() -> TestResult {

        let id_prefixes = vec!["pref1".to_string(), "pref2".to_string()];
        let f = SubscriptionFilter::new().ids(id_prefixes.clone());

        assert_eq!(
            Some(id_prefixes),
            f.ids
        );

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
        let handled_valid_eose_msg =
            RelayMessage::new_eose(String::from("random-subscription-id"));

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
        let pk =
            XOnlyPublicKey::from_str("379e863e8357163b5bce5d2688dc4f1dcc2d505222fb8d74db600f30535dfdfe")
                .unwrap();
        let filters = vec![
            SubscriptionFilter::new().kind_base(KindBase::EncryptedDirectMessage),
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
        let pk =
            XOnlyPublicKey::from_str("379e863e8357163b5bce5d2688dc4f1dcc2d505222fb8d74db600f30535dfdfe")
                .unwrap();
        let filters = vec![
            SubscriptionFilter::new().kind_custom(22),
            SubscriptionFilter::new().pubkey(pk),
        ];

        let client_req = ClientMessage::new_req("test", filters);
        assert_eq!(
            client_req.to_json(),
            r##"["REQ","test",{"kinds":[22]},{"#p":["379e863e8357163b5bce5d2688dc4f1dcc2d505222fb8d74db600f30535dfdfe"]}]"##
        );
    }
}
