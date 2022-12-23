// Copyright (c) 2021 Paul Miller
// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use serde_json::{json, Value};

use crate::{Event, Sha256Hash};

#[derive(Debug, Eq, PartialEq, thiserror::Error)]
pub enum MessageHandleError {
    #[error("Message has an invalid format")]
    InvalidMessageFormat,
    #[error("Json deserialization failed")]
    JsonDeserializationFailed,
}

/// Messages sent by relays, received by clients
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum RelayMessage {
    Event {
        subscription_id: String,
        event: Box<Event>,
    },
    Notice {
        message: String,
    },
    EndOfStoredEvents {
        subscription_id: String,
    },
    Ok {
        event_id: Sha256Hash,
        status: bool,
        message: String,
    },
    Empty,
}

impl RelayMessage {
    // Relay is responsible for storing corresponding subscription id
    pub fn new_event(subscription_id: String, event: Event) -> Self {
        Self::Event {
            subscription_id,
            event: Box::new(event),
        }
    }

    pub fn new_notice(message: String) -> Self {
        Self::Notice { message }
    }

    pub fn new_eose(subscription_id: String) -> Self {
        Self::EndOfStoredEvents { subscription_id }
    }

    pub fn new_ok(event_id: Sha256Hash, status: bool, message: String) -> Self {
        Self::Ok {
            event_id,
            status,
            message,
        }
    }

    pub fn to_json(&self) -> String {
        match self {
            Self::Event {
                event,
                subscription_id,
            } => json!(["EVENT", subscription_id, event]).to_string(),
            Self::Notice { message } => json!(["NOTICE", message]).to_string(),
            Self::EndOfStoredEvents { subscription_id } => {
                json!(["EOSE", subscription_id]).to_string()
            }
            Self::Ok {
                event_id,
                status,
                message,
            } => json!(["OK", event_id, status, message]).to_string(),
            Self::Empty => String::new(),
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

            let subscription_id: String = serde_json::from_value(v[1].clone())
                .map_err(|_| MessageHandleError::JsonDeserializationFailed)?;
            let event = Event::from_json(v[2].to_string())
                .map_err(|_| MessageHandleError::JsonDeserializationFailed)?;

            return Ok(Self::new_event(subscription_id, event));
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

        // OK (NIP-20)
        // Relay response format: ["OK", <event_id>, <true|false>, <message>]
        if v[0] == "OK" {
            if v.len() != 4 {
                return Err(MessageHandleError::InvalidMessageFormat);
            }

            let event_id: Sha256Hash = serde_json::from_value(v[1].clone())
                .map_err(|_| MessageHandleError::JsonDeserializationFailed)?;

            let status: bool = serde_json::from_value(v[2].clone())
                .map_err(|_| MessageHandleError::JsonDeserializationFailed)?;

            let message: String = serde_json::from_value(v[3].clone())
                .map_err(|_| MessageHandleError::JsonDeserializationFailed)?;

            return Ok(Self::new_ok(event_id, status, message));
        }

        Err(MessageHandleError::InvalidMessageFormat)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use crate::Result;

    #[test]
    fn test_handle_valid_notice() -> Result<()> {
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
    fn test_handle_valid_event() -> Result<()> {
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
            RelayMessage::new_event("random_string".to_string(), handled_event?)
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
    fn test_handle_valid_eose() -> Result<()> {
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
    fn test_handle_valid_ok() -> Result<()> {
        let valid_ok_msg = r#"["OK", "b1a649ebe8b435ec71d3784793f3bbf4b93e64e17568a741aecd4c7ddeafce30", true, "pow: difficulty 25>=24"]"#;
        let handled_valid_ok_msg = RelayMessage::new_ok(
            Sha256Hash::from_str(
                "b1a649ebe8b435ec71d3784793f3bbf4b93e64e17568a741aecd4c7ddeafce30",
            )?,
            true,
            "pow: difficulty 25>=24".into(),
        );

        assert_eq!(RelayMessage::from_json(valid_ok_msg)?, handled_valid_ok_msg);

        Ok(())
    }
    #[test]
    fn test_handle_invalid_ok() {
        // Missing params
        assert_eq!(
            RelayMessage::from_json(
                r#"["OK", "b1a649ebe8b435ec71d3784793f3bbf4b93e64e17568a741aecd4c7ddeafce30"]"#
            )
            .unwrap_err(),
            MessageHandleError::InvalidMessageFormat
        );

        // Invalid event_id
        assert_eq!(
            RelayMessage::from_json(
                r#"["OK", "b1a649ebe8b435ec71d3784793f3bbf4b93e64e17568a741aecd4c7dde", true, ""]"#
            )
            .unwrap_err(),
            MessageHandleError::JsonDeserializationFailed
        );

        // Invalid status
        assert_eq!(
            RelayMessage::from_json(r#"["OK", "b1a649ebe8b435ec71d3784793f3bbf4b93e64e17568a741aecd4c7ddeafce30", hello, ""]"#).unwrap_err(),
            MessageHandleError::JsonDeserializationFailed
        );

        // Invalid message
        assert_eq!(
            RelayMessage::from_json(r#"["OK", "b1a649ebe8b435ec71d3784793f3bbf4b93e64e17568a741aecd4c7ddeafce30", hello, 404]"#).unwrap_err(),
            MessageHandleError::JsonDeserializationFailed
        );
    }
}
