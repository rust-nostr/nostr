use crate::{Event, Kind};
use chrono::{DateTime, Utc};
use secp256k1::schnorrsig::PublicKey;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use thiserror::Error;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct SubscriptionFilter {
    // authors: Vec<PublicKey>,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    kind: Option<Kind>,
    // #e
    #[serde(rename = "#e")]
    #[serde(skip_serializing_if = "Option::is_none")]
    tag_e: Option<String>,
    // #p, for instance the receiver public key
    #[serde(rename = "#p")]
    #[serde(skip_serializing_if = "Option::is_none")]
    tag_p: Option<PublicKey>,
    // TODO: we can't skip this always
    // #[serde(with = "ts_seconds")]
    // #[serde(skip_deserializing)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(skip)]
    since: Option<DateTime<Utc>>, // unix timestamp seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    authors: Option<Vec<PublicKey>>,
}

impl SubscriptionFilter {
    pub fn new() -> Self {
        Self {
            id: None,
            kind: None,
            tag_e: None,
            tag_p: None,
            since: None,
            authors: None,
        }
    }

    pub fn id(self, id: impl Into<String>) -> Self {
        Self {
            id: Some(id.into()),
            ..self
        }
    }

    // kind: Option<usize>,
    pub fn kind(self, kind: Kind) -> Self {
        Self {
            kind: Some(kind),
            ..self
        }
    }
    // // #e
    // #[serde(rename = "#e")]
    // tag_e: Option<String>,
    pub fn tag_e(self, event_id: impl Into<String>) -> Self {
        Self {
            tag_e: Some(event_id.into()),
            ..self
        }
    }
    // // #p, for instance the receiver public key
    // #[serde(rename = "#p")]
    // tag_p: Option<PublicKey>,
    pub fn tag_p(self, pubkey: PublicKey) -> Self {
        Self {
            tag_p: Some(pubkey),
            ..self
        }
    }
    // #[serde(with = "ts_seconds")]
    // since: Option<DateTime<Utc>>, // unix timestamp seconds
    pub fn since(self, since: DateTime<Utc>) -> Self {
        Self {
            since: Some(since),
            ..self
        }
    }
    // authors: Option<Vec<PublicKey>>,
    pub fn authors(self, authors: Vec<PublicKey>) -> Self {
        Self {
            authors: Some(authors),
            ..self
        }
    }

    // pub fn new(authors: Vec<PublicKey>) -> Self {
    //     SubscriptionFilter { author: authors[0] }
    // }
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
    pub fn to_json(&self) -> String {
        match self {
            Self::Empty => String::new(),
            Self::Event {
                event,
                subscription_id,
            } => json!(["EVENT", subscription_id, event]).to_string(),
            Self::Notice { message } => json!(["NOTICE", message]).to_string(),
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
        filter: SubscriptionFilter,
    },
    Close {
        subscription_id: String,
    },
}

impl ClientMessage {
    pub fn new_event(event: Event) -> Self {
        Self::Event { event }
    }

    pub fn new_req(subscription_id: impl Into<String>, filter: SubscriptionFilter) -> Self {
        Self::Req {
            subscription_id: subscription_id.into(),
            filter,
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
                filter,
            } => json!(["REQ", subscription_id, filter]).to_string(),
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
    use std::error::Error;

    type TestResult = Result<(), Box<dyn Error>>;

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
}
