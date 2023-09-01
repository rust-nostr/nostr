// Copyright (c) 2021 Paul Miller
// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Client messages

use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use bitcoin::secp256k1::{Secp256k1, Verification};
#[cfg(feature = "std")]
use serde::{Deserialize, Deserializer};
use serde::{Serialize, Serializer};
use serde_json::{json, Value};

use super::{Filter, MessageHandleError, SubscriptionId};
use crate::Event;
#[cfg(feature = "std")]
use crate::SECP256K1;

/// Messages sent by clients, received by relays
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClientMessage {
    /// Event
    Event(Box<Event>),
    /// Req
    Req {
        /// Subscription ID
        subscription_id: SubscriptionId,
        /// Filters
        filters: Vec<Filter>,
    },
    /// Count
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/45.md>
    Count {
        /// Subscription ID
        subscription_id: SubscriptionId,
        /// Filters
        filters: Vec<Filter>,
    },
    /// Close
    Close(SubscriptionId),
    /// Auth
    Auth(Box<Event>),
}

impl Serialize for ClientMessage {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let json_value: Value = self.as_value();
        json_value.serialize(serializer)
    }
}

#[cfg(feature = "std")]
impl<'de> Deserialize<'de> for ClientMessage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let json_value = Value::deserialize(deserializer)?;
        ClientMessage::from_value(json_value).map_err(serde::de::Error::custom)
    }
}

impl ClientMessage {
    /// Create new `EVENT` message
    pub fn new_event(event: Event) -> Self {
        Self::Event(Box::new(event))
    }

    /// Create new `REQ` message
    pub fn new_req(subscription_id: SubscriptionId, filters: Vec<Filter>) -> Self {
        Self::Req {
            subscription_id,
            filters,
        }
    }

    /// Create new `COUNT` message
    pub fn new_count(subscription_id: SubscriptionId, filters: Vec<Filter>) -> Self {
        Self::Count {
            subscription_id,
            filters,
        }
    }

    /// Create new `CLOSE` message
    pub fn close(subscription_id: SubscriptionId) -> Self {
        Self::Close(subscription_id)
    }

    /// Create new `AUTH` message
    pub fn new_auth(event: Event) -> Self {
        Self::Auth(Box::new(event))
    }

    /// Check if is an `EVENT` message
    pub fn is_event(&self) -> bool {
        matches!(self, ClientMessage::Event(_))
    }

    /// Check if is an `REQ` message
    pub fn is_req(&self) -> bool {
        matches!(self, ClientMessage::Req { .. })
    }

    /// Check if is an `CLOSE` message
    pub fn is_close(&self) -> bool {
        matches!(self, ClientMessage::Close(_))
    }

    /// Serialize as [`Value`]
    pub fn as_value(&self) -> Value {
        match self {
            Self::Event(event) => json!(["EVENT", event]),
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

                json
            }
            Self::Count {
                subscription_id,
                filters,
            } => {
                let mut json = json!(["COUNT", subscription_id]);
                let mut filters = json!(filters);

                if let Some(json) = json.as_array_mut() {
                    if let Some(filters) = filters.as_array_mut() {
                        json.append(filters);
                    }
                }

                json
            }
            Self::Close(subscription_id) => json!(["CLOSE", subscription_id]),
            Self::Auth(event) => json!(["AUTH", event]),
        }
    }

    /// Serialize [`ClientMessage`] as JSON string
    pub fn as_json(&self) -> String {
        self.as_value().to_string()
    }

    /// Deserialize from [`Value`]
    #[cfg(feature = "std")]
    pub fn from_value(msg: Value) -> Result<Self, MessageHandleError> {
        Self::from_value_with_ctx(&SECP256K1, msg)
    }

    /// Deserialize from [`Value`]
    pub fn from_value_with_ctx<C>(
        secp: &Secp256k1<C>,
        msg: Value,
    ) -> Result<Self, MessageHandleError>
    where
        C: Verification,
    {
        let v = msg
            .as_array()
            .ok_or(MessageHandleError::InvalidMessageFormat)?;

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
            let event = Event::from_json_with_ctx(secp, v[1].to_string())?;
            return Ok(Self::new_event(event));
        }

        // Req
        // ["REQ", <subscription_id>, <filter JSON>, <filter JSON>...]
        if v[0] == "REQ" {
            if v_len == 2 {
                let subscription_id: SubscriptionId = serde_json::from_value(v[1].clone())?;
                return Ok(Self::new_req(subscription_id, Vec::new()));
            } else if v_len >= 3 {
                let subscription_id: SubscriptionId = serde_json::from_value(v[1].clone())?;
                let filters: Vec<Filter> = serde_json::from_value(Value::Array(v[2..].to_vec()))?;
                return Ok(Self::new_req(subscription_id, filters));
            } else {
                return Err(MessageHandleError::InvalidMessageFormat);
            }
        }

        // ["COUNT", <subscription_id>, <filter JSON>, <filter JSON>...]
        if v[0] == "COUNT" {
            if v_len == 2 {
                let subscription_id: SubscriptionId = serde_json::from_value(v[1].clone())?;
                return Ok(Self::new_count(subscription_id, Vec::new()));
            } else if v_len >= 3 {
                let subscription_id: SubscriptionId = serde_json::from_value(v[1].clone())?;
                let filters: Vec<Filter> = serde_json::from_value(Value::Array(v[2..].to_vec()))?;
                return Ok(Self::new_count(subscription_id, filters));
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

            let subscription_id: SubscriptionId = serde_json::from_value(v[1].clone())?;

            return Ok(Self::close(subscription_id));
        }

        // Auth
        // ["AUTH", <event JSON>]
        if v[0] == "AUTH" {
            if v_len != 2 {
                return Err(MessageHandleError::InvalidMessageFormat);
            }
            let event = Event::from_json_with_ctx(secp, v[1].to_string())?;
            return Ok(Self::new_auth(event));
        }

        Err(MessageHandleError::InvalidMessageFormat)
    }

    /// Deserialize [`ClientMessage`] from JSON string
    #[cfg(feature = "std")]
    pub fn from_json<S>(msg: S) -> Result<Self, MessageHandleError>
    where
        S: Into<String>,
    {
        Self::from_json_with_ctx(&SECP256K1, msg)
    }

    /// Deserialize [`ClientMessage`] from JSON string
    pub fn from_json_with_ctx<C, S>(secp: &Secp256k1<C>, msg: S) -> Result<Self, MessageHandleError>
    where
        C: Verification,
        S: Into<String>,
    {
        let msg: &str = &msg.into();
        tracing::trace!("{}", msg);

        if msg.is_empty() {
            return Err(MessageHandleError::InvalidMessageFormat);
        }

        let value: Value = serde_json::from_str(msg)?;
        Self::from_value_with_ctx(secp, value)
    }
}

#[cfg(test)]
mod tests {

    use core::str::FromStr;

    use bitcoin::secp256k1::XOnlyPublicKey;

    use super::*;
    use crate::Kind;

    #[test]
    fn test_client_message_req() {
        let pk = XOnlyPublicKey::from_str(
            "379e863e8357163b5bce5d2688dc4f1dcc2d505222fb8d74db600f30535dfdfe",
        )
        .unwrap();
        let filters = vec![
            Filter::new().kind(Kind::EncryptedDirectMessage),
            Filter::new().pubkey(pk),
        ];

        let client_req = ClientMessage::new_req(SubscriptionId::new("test"), filters);
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
            Filter::new().kind(Kind::Custom(22)),
            Filter::new().pubkey(pk),
        ];

        let client_req = ClientMessage::new_req(SubscriptionId::new("test"), filters);
        assert_eq!(
            client_req.as_json(),
            r##"["REQ","test",{"kinds":[22]},{"#p":["379e863e8357163b5bce5d2688dc4f1dcc2d505222fb8d74db600f30535dfdfe"]}]"##
        );
    }

    #[test]
    fn test_negative_timestamp() {
        let secp = Secp256k1::new();

        let req = json!([
            "REQ",
            "some_id",
            {
                "authors": [
                    "379e863e8357163b5bce5d2688dc4f1dcc2d505222fb8d74db600f30535dfdfe"
                ],
                "kinds": [
                    0,
                    3
                ],
                "limit": 200000,
                "since": -50123406
            }
        ]);

        let msg = ClientMessage::from_value_with_ctx(&secp, req.clone()).unwrap();

        assert_eq!(msg.as_value(), req)
    }
}
