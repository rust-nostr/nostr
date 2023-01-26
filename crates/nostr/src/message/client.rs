// Copyright (c) 2021 Paul Miller
// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Client messages

use serde_json::{json, Value};

use super::MessageHandleError;
use crate::{Event, SubscriptionFilter};

/// Messages sent by clients, received by relays
#[allow(missing_docs)]
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ClientMessage {
    /// Event
    Event { event: Box<Event> },
    /// Req
    Req {
        subscription_id: String,
        filters: Vec<SubscriptionFilter>,
    },
    /// Close
    Close { subscription_id: String },
    /// Auth
    Auth { event: Box<Event> },
}

impl ClientMessage {
    /// Create new `EVENT` message
    pub fn new_event(event: Event) -> Self {
        Self::Event {
            event: Box::new(event),
        }
    }

    /// Create new `REQ` message
    pub fn new_req<S>(subscription_id: S, filters: Vec<SubscriptionFilter>) -> Self
    where
        S: Into<String>,
    {
        Self::Req {
            subscription_id: subscription_id.into(),
            filters,
        }
    }

    /// Create new `CLOSE` message
    pub fn close<S>(subscription_id: S) -> Self
    where
        S: Into<String>,
    {
        Self::Close {
            subscription_id: subscription_id.into(),
        }
    }

    /// Create new `AUTH` message
    pub fn new_auth(event: Event) -> Self {
        Self::Auth {
            event: Box::new(event),
        }
    }

    /// Serialize [`ClientMessage`] as JSON string
    pub fn as_json(&self) -> String {
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
            Self::Auth { event } => json!(["AUTH", event]).to_string(),
        }
    }

    /// Deserialize [`ClientMessage`] from JSON string
    pub fn from_json<S>(msg: S) -> Result<Self, MessageHandleError>
    where
        S: Into<String>,
    {
        let msg: &str = &msg.into();

        log::trace!("{}", msg);

        let v: Vec<Value> =
            serde_json::from_str(msg).map_err(|_| MessageHandleError::JsonDeserializationFailed)?;

        if v.is_empty() {
            return Err(MessageHandleError::InvalidMessageFormat);
        }

        let v_len: usize = v.len();

        // Event
        // ["EVENT", <event JSON>]
        if v[0] == "EVENT" {
            if v_len != 2 {
                return Err(MessageHandleError::InvalidMessageFormat);
            }
            let event = Event::from_json(v[1].to_string())
                .map_err(|_| MessageHandleError::JsonDeserializationFailed)?;
            return Ok(Self::new_event(event));
        }

        // Req
        // ["REQ", <subscription_id>, <filter JSON>, <filter JSON>...]
        if v[0] == "REQ" {
            if v_len == 2 {
                let subscription_id: String = serde_json::from_value(v[1].clone())
                    .map_err(|_| MessageHandleError::JsonDeserializationFailed)?;
                return Ok(Self::new_req(subscription_id, Vec::new()));
            } else if v_len == 3 {
                let subscription_id: String = serde_json::from_value(v[1].clone())
                    .map_err(|_| MessageHandleError::JsonDeserializationFailed)?;
                let filters: Vec<SubscriptionFilter> =
                    serde_json::from_value(Value::Array(v[2..].to_vec()))
                        .map_err(|_| MessageHandleError::JsonDeserializationFailed)?;
                return Ok(Self::new_req(subscription_id, filters));
            } else {
                return Err(MessageHandleError::InvalidMessageFormat);
            }
        }

        // Close
        // ["CLOSE", <subscription_id>]
        if v[0] == "CLOSE" {
            if v_len != 2 {
                return Err(MessageHandleError::InvalidMessageFormat);
            }

            let subscription_id: String = serde_json::from_value(v[1].clone())
                .map_err(|_| MessageHandleError::JsonDeserializationFailed)?;

            return Ok(Self::close(subscription_id));
        }

        // Auth
        // ["AUTH", <event JSON>]
        if v[0] == "AUTH" {
            if v_len != 2 {
                return Err(MessageHandleError::InvalidMessageFormat);
            }
            let event = Event::from_json(v[1].to_string())
                .map_err(|_| MessageHandleError::JsonDeserializationFailed)?;
            return Ok(Self::new_auth(event));
        }

        Err(MessageHandleError::InvalidMessageFormat)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::str::FromStr;

    use bitcoin::secp256k1::XOnlyPublicKey;

    use crate::Kind;

    #[test]
    fn test_client_message_req() {
        let pk = XOnlyPublicKey::from_str(
            "379e863e8357163b5bce5d2688dc4f1dcc2d505222fb8d74db600f30535dfdfe",
        )
        .unwrap();
        let filters = vec![
            SubscriptionFilter::new().kind(Kind::EncryptedDirectMessage),
            SubscriptionFilter::new().pubkey(pk),
        ];

        let client_req = ClientMessage::new_req("test", filters);
        assert_eq!(
            client_req.as_json(),
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
            client_req.as_json(),
            r##"["REQ","test",{"kinds":[22]},{"#p":["379e863e8357163b5bce5d2688dc4f1dcc2d505222fb8d74db600f30535dfdfe"]}]"##
        );
    }
}
