// Copyright (c) 2021 Paul Miller
// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Relay messages

use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{json, Value};

use super::MessageHandleError;
use crate::{Event, EventId, SubscriptionId};

/// Messages sent by relays, received by clients
#[allow(missing_docs)]
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum RelayMessage {
    Event {
        subscription_id: SubscriptionId,
        event: Box<Event>,
    },
    Notice {
        message: String,
    },
    EndOfStoredEvents(SubscriptionId),
    Ok {
        event_id: EventId,
        status: bool,
        message: String,
    },
    Auth {
        challenge: String,
    },
    Empty,
}

impl Serialize for RelayMessage {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let json_value: Value = self.as_value();
        json_value.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for RelayMessage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let json_value = Value::deserialize(deserializer)?;
        RelayMessage::from_value(json_value).map_err(Error::custom)
    }
}

impl RelayMessage {
    /// Create new `EVENT` message
    pub fn new_event(subscription_id: SubscriptionId, event: Event) -> Self {
        Self::Event {
            subscription_id,
            event: Box::new(event),
        }
    }

    /// Create new `NOTICE` message
    pub fn new_notice<S>(message: S) -> Self
    where
        S: Into<String>,
    {
        Self::Notice {
            message: message.into(),
        }
    }

    /// Create new `EOSE` message
    pub fn new_eose(subscription_id: SubscriptionId) -> Self {
        Self::EndOfStoredEvents(subscription_id)
    }

    /// Create new `OK` message
    pub fn new_ok<S>(event_id: EventId, status: bool, message: S) -> Self
    where
        S: Into<String>,
    {
        Self::Ok {
            event_id,
            status,
            message: message.into(),
        }
    }

    /// Create new `AUTH` message
    pub fn new_auth<S>(challenge: S) -> Self
    where
        S: Into<String>,
    {
        Self::Auth {
            challenge: challenge.into(),
        }
    }

    fn as_value(&self) -> Value {
        match self {
            Self::Event {
                event,
                subscription_id,
            } => json!(["EVENT", subscription_id, event]),
            Self::Notice { message } => json!(["NOTICE", message]),
            Self::EndOfStoredEvents(subscription_id) => {
                json!(["EOSE", subscription_id])
            }
            Self::Ok {
                event_id,
                status,
                message,
            } => json!(["OK", event_id, status, message]),
            Self::Auth { challenge } => json!(["AUTH", challenge]),
            Self::Empty => Value::Null,
        }
    }

    /// Serialize [`RelayMessage`] as JSON string
    pub fn as_json(&self) -> String {
        self.as_value().to_string()
    }

    /// Deserialize [`RelayMessage`] from [`Value`]
    fn from_value(msg: Value) -> Result<Self, MessageHandleError> {
        let v = msg
            .as_array()
            .ok_or(MessageHandleError::InvalidMessageFormat)?;

        if v.is_empty() {
            return Err(MessageHandleError::InvalidMessageFormat);
        }

        let v_len: usize = v.len();

        // Notice
        // Relay response format: ["NOTICE", <message>]
        if v[0] == "NOTICE" {
            if v_len != 2 {
                return Err(MessageHandleError::InvalidMessageFormat);
            }
            let v_notice: String = serde_json::from_value(v[1].clone())?;
            return Ok(Self::Notice { message: v_notice });
        }

        // Event
        // Relay response format: ["EVENT", <subscription id>, <event JSON>]
        if v[0] == "EVENT" {
            if v_len != 3 {
                return Err(MessageHandleError::InvalidMessageFormat);
            }

            let subscription_id: SubscriptionId = serde_json::from_value(v[1].clone())?;
            let event = Event::from_json(v[2].to_string())?;

            return Ok(Self::new_event(subscription_id, event));
        }

        // EOSE (NIP-15)
        // Relay response format: ["EOSE", <subscription_id>]
        if v[0] == "EOSE" {
            if v_len != 2 {
                return Err(MessageHandleError::InvalidMessageFormat);
            }

            let subscription_id: SubscriptionId = serde_json::from_value(v[1].clone())?;

            return Ok(Self::new_eose(subscription_id));
        }

        // OK (NIP-20)
        // Relay response format: ["OK", <event_id>, <true|false>, <message>]
        if v[0] == "OK" {
            if v_len != 4 {
                return Err(MessageHandleError::InvalidMessageFormat);
            }

            let event_id: EventId = serde_json::from_value(v[1].clone())?;

            let status: bool = serde_json::from_value(v[2].clone())?;

            let message: String = serde_json::from_value(v[3].clone())?;

            return Ok(Self::new_ok(event_id, status, message));
        }

        Err(MessageHandleError::InvalidMessageFormat)
    }

    /// Deserialize [`RelayMessage`] as JSON string
    pub fn from_json<S>(msg: S) -> Result<Self, MessageHandleError>
    where
        S: Into<String>,
    {
        let msg: &str = &msg.into();
        log::trace!("{}", msg);

        if msg.is_empty() {
            return Err(MessageHandleError::InvalidMessageFormat);
        }

        let value: Value = serde_json::from_str(msg)?;
        Self::from_value(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Result, Timestamp};

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

        assert!(RelayMessage::from_json(invalid_notice_msg).is_err(),);
        assert!(RelayMessage::from_json(invalid_notice_msg_content).is_err(),);
    }

    #[test]
    fn test_handle_valid_event() -> Result<()> {
        let valid_event_msg = r#"["EVENT", "random_string", {"id":"70b10f70c1318967eddf12527799411b1a9780ad9c43858f5e5fcd45486a13a5","pubkey":"379e863e8357163b5bce5d2688dc4f1dcc2d505222fb8d74db600f30535dfdfe","created_at":1612809991,"kind":1,"tags":[],"content":"test","sig":"273a9cd5d11455590f4359500bccb7a89428262b96b3ea87a756b770964472f8c3e87f5d5e64d8d2e859a71462a3f477b554565c4f2f326cb01dd7620db71502"}]"#;

        let id = "70b10f70c1318967eddf12527799411b1a9780ad9c43858f5e5fcd45486a13a5";
        let pubkey = "379e863e8357163b5bce5d2688dc4f1dcc2d505222fb8d74db600f30535dfdfe";
        let created_at = Timestamp::from(1612809991);
        let kind = 1;
        let tags = vec![];
        let content = "test";
        let sig = "273a9cd5d11455590f4359500bccb7a89428262b96b3ea87a756b770964472f8c3e87f5d5e64d8d2e859a71462a3f477b554565c4f2f326cb01dd7620db71502";

        let handled_event = Event::new_dummy(id, pubkey, created_at, kind, tags, content, sig);

        assert_eq!(
            RelayMessage::from_json(valid_event_msg)?,
            RelayMessage::new_event(SubscriptionId::new("random_string"), handled_event?)
        );

        Ok(())
    }

    #[test]
    fn test_handle_invalid_event() {
        //Mising Event field
        let invalid_event_msg = r#"["EVENT", "random_string"]"#;
        //Event JSON with incomplete content
        let invalid_event_msg_content = r#"["EVENT", "random_string", {"id":"70b10f70c1318967eddf12527799411b1a9780ad9c43858f5e5fcd45486a13a5","pubkey":"379e863e8357163b5bce5d2688dc4f1dcc2d505222fb8d74db600f30535dfdfe"}]"#;

        assert!(RelayMessage::from_json(invalid_event_msg).is_err(),);

        assert!(RelayMessage::from_json(invalid_event_msg_content).is_err(),);
    }

    #[test]
    fn test_handle_valid_eose() -> Result<()> {
        let valid_eose_msg = r#"["EOSE","random-subscription-id"]"#;
        let handled_valid_eose_msg =
            RelayMessage::new_eose(SubscriptionId::new("random-subscription-id"));

        assert_eq!(
            RelayMessage::from_json(valid_eose_msg)?,
            handled_valid_eose_msg
        );

        Ok(())
    }
    #[test]
    fn test_handle_invalid_eose() {
        // Missing subscription ID
        assert!(RelayMessage::from_json(r#"["EOSE"]"#).is_err(),);

        // The subscription ID is not string
        assert!(RelayMessage::from_json(r#"["EOSE", 404]"#).is_err(),);
    }

    #[test]
    fn test_handle_valid_ok() -> Result<()> {
        let valid_ok_msg = r#"["OK", "b1a649ebe8b435ec71d3784793f3bbf4b93e64e17568a741aecd4c7ddeafce30", true, "pow: difficulty 25>=24"]"#;
        let handled_valid_ok_msg = RelayMessage::new_ok(
            EventId::from_hex("b1a649ebe8b435ec71d3784793f3bbf4b93e64e17568a741aecd4c7ddeafce30")?,
            true,
            "pow: difficulty 25>=24",
        );

        assert_eq!(RelayMessage::from_json(valid_ok_msg)?, handled_valid_ok_msg);

        Ok(())
    }
    #[test]
    fn test_handle_invalid_ok() {
        // Missing params
        assert!(RelayMessage::from_json(
            r#"["OK", "b1a649ebe8b435ec71d3784793f3bbf4b93e64e17568a741aecd4c7ddeafce30"]"#
        )
        .is_err(),);

        // Invalid event_id
        assert!(RelayMessage::from_json(
            r#"["OK", "b1a649ebe8b435ec71d3784793f3bbf4b93e64e17568a741aecd4c7dde", true, ""]"#
        )
        .is_err(),);

        // Invalid status
        assert!(
            RelayMessage::from_json(r#"["OK", "b1a649ebe8b435ec71d3784793f3bbf4b93e64e17568a741aecd4c7ddeafce30", hello, ""]"#).is_err(),
        );

        // Invalid message
        assert!(
            RelayMessage::from_json(r#"["OK", "b1a649ebe8b435ec71d3784793f3bbf4b93e64e17568a741aecd4c7ddeafce30", hello, 404]"#).is_err()
        );
    }

    #[test]
    fn parse_message() -> Result<()> {
        // Got this fresh off the wire
        pub const SAMPLE_EVENT: &'static str = r#"["EVENT", "random_string", {"id":"70b10f70c1318967eddf12527799411b1a9780ad9c43858f5e5fcd45486a13a5","pubkey":"379e863e8357163b5bce5d2688dc4f1dcc2d505222fb8d74db600f30535dfdfe","created_at":1612809991,"kind":1,"tags":[],"content":"test","sig":"273a9cd5d11455590f4359500bccb7a89428262b96b3ea87a756b770964472f8c3e87f5d5e64d8d2e859a71462a3f477b554565c4f2f326cb01dd7620db71502"}]"#;

        // Hand parsed version as a sanity check
        let id = "70b10f70c1318967eddf12527799411b1a9780ad9c43858f5e5fcd45486a13a5";
        let pubkey = "379e863e8357163b5bce5d2688dc4f1dcc2d505222fb8d74db600f30535dfdfe";
        let created_at = Timestamp::from(1612809991);
        let kind = 1;
        let tags = vec![];
        let content = "test";
        let sig = "273a9cd5d11455590f4359500bccb7a89428262b96b3ea87a756b770964472f8c3e87f5d5e64d8d2e859a71462a3f477b554565c4f2f326cb01dd7620db71502";

        let event = Event::new_dummy(id, pubkey, created_at, kind, tags, content, sig);

        let parsed_event = RelayMessage::from_json(SAMPLE_EVENT);

        assert_eq!(
            parsed_event.expect("Failed to parse event"),
            RelayMessage::new_event(SubscriptionId::new("random_string"), event?)
        );

        Ok(())
    }
}
