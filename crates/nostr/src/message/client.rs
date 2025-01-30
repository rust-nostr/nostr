// Copyright (c) 2021 Paul Miller
// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Client messages

use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{json, Value};

use super::MessageHandleError;
use crate::{Event, Filter, JsonUtil, SubscriptionId};

/// Messages sent by clients, received by relays
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClientMessage {
    /// Event
    Event(Box<Event>),
    /// Req
    Req {
        /// Subscription ID
        subscription_id: SubscriptionId,
        /// Filter
        filter: Box<Filter>,
    },
    /// Multi-filter REQ (deprecated)
    ///
    /// <https://github.com/nostr-protocol/nips/pull/1645>
    ReqMultiFilter {
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
        /// Filter
        filter: Box<Filter>,
    },
    /// Close
    Close(SubscriptionId),
    /// Auth
    Auth(Box<Event>),
    /// Negentropy Open
    NegOpen {
        /// Subscription ID
        subscription_id: SubscriptionId,
        /// Filter
        filter: Box<Filter>,
        /// ID size (deprecated)
        id_size: Option<u8>,
        /// Initial message (hex)
        initial_message: String,
    },
    /// Negentropy Message
    NegMsg {
        /// Subscription ID
        subscription_id: SubscriptionId,
        /// Message
        message: String,
    },
    /// Negentropy Close
    NegClose {
        /// Subscription ID
        subscription_id: SubscriptionId,
    },
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
    /// Create `EVENT` message
    #[inline]
    pub fn event(event: Event) -> Self {
        Self::Event(Box::new(event))
    }

    /// Create `REQ` message
    #[inline]
    pub fn req(subscription_id: SubscriptionId, filter: Filter) -> Self {
        Self::Req {
            subscription_id,
            filter: Box::new(filter),
        }
    }

    /// Create `COUNT` message
    #[inline]
    pub fn count(subscription_id: SubscriptionId, filter: Filter) -> Self {
        Self::Count {
            subscription_id,
            filter: Box::new(filter),
        }
    }

    /// Create new `CLOSE` message
    #[inline]
    pub fn close(subscription_id: SubscriptionId) -> Self {
        Self::Close(subscription_id)
    }

    /// Create `AUTH` message
    #[inline]
    pub fn auth(event: Event) -> Self {
        Self::Auth(Box::new(event))
    }

    /// Create new `NEG-OPEN` message
    pub fn neg_open(
        subscription_id: SubscriptionId,
        filter: Filter,
        initial_message: String,
    ) -> Self {
        Self::NegOpen {
            subscription_id,
            filter: Box::new(filter),
            id_size: None,
            initial_message,
        }
    }

    /// Check if is an `EVENT` message
    #[inline]
    pub fn is_event(&self) -> bool {
        matches!(self, ClientMessage::Event(_))
    }

    /// Check if is an `REQ` message
    #[inline]
    pub fn is_req(&self) -> bool {
        matches!(self, ClientMessage::Req { .. })
    }

    /// Check if is an `CLOSE` message
    #[inline]
    pub fn is_close(&self) -> bool {
        matches!(self, ClientMessage::Close(_))
    }

    /// Check if is an `AUTH` message
    #[inline]
    pub fn is_auth(&self) -> bool {
        matches!(self, ClientMessage::Auth(_))
    }

    /// Serialize as [`Value`]
    pub fn as_value(&self) -> Value {
        match self {
            Self::Event(event) => json!(["EVENT", event]),
            Self::Req {
                subscription_id,
                filter,
            } => {
                json!(["REQ", subscription_id, filter])
            }
            Self::ReqMultiFilter {
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
                filter,
            } => {
                json!(["COUNT", subscription_id, filter])
            }
            Self::Close(subscription_id) => json!(["CLOSE", subscription_id]),
            Self::Auth(event) => json!(["AUTH", event]),
            Self::NegOpen {
                subscription_id,
                filter,
                id_size,
                initial_message,
            } => match id_size {
                Some(id_size) => json!([
                    "NEG-OPEN",
                    subscription_id,
                    filter,
                    id_size,
                    initial_message
                ]),
                None => json!(["NEG-OPEN", subscription_id, filter, initial_message]),
            },
            Self::NegMsg {
                subscription_id,
                message,
            } => json!(["NEG-MSG", subscription_id, message]),
            Self::NegClose { subscription_id } => json!(["NEG-CLOSE", subscription_id]),
        }
    }

    /// Deserialize from [`Value`]
    ///
    /// **This method NOT verify the event signature!**
    pub fn from_value(msg: Value) -> Result<Self, MessageHandleError> {
        let v = msg
            .as_array()
            .ok_or(MessageHandleError::InvalidMessageFormat)?;

        if v.is_empty() {
            return Err(MessageHandleError::InvalidMessageFormat);
        }

        let v_len: usize = v.len();

        // ["EVENT", <event JSON>]
        if v[0] == "EVENT" {
            if v_len >= 2 {
                let event: Event = serde_json::from_value(v[1].clone())?;
                return Ok(Self::event(event));
            } else {
                return Err(MessageHandleError::InvalidMessageFormat);
            }
        }

        // ["REQ", <subscription_id>, <filter JSON>]
        if v[0] == "REQ" {
            if v_len == 3 {
                let subscription_id: SubscriptionId = serde_json::from_value(v[1].clone())?;
                let filter: Filter = serde_json::from_value(v[2].clone())?;
                return Ok(Self::req(subscription_id, filter));
            } else if v_len >= 3 {
                // Deprecated REQ
                // ["REQ", <subscription_id>, <filter JSON>, <filter JSON>, ...]
                let subscription_id: SubscriptionId = serde_json::from_value(v[1].clone())?;
                let filters: Vec<Filter> = serde_json::from_value(Value::Array(v[2..].to_vec()))?;
                return Ok(Self::ReqMultiFilter {
                    subscription_id,
                    filters,
                });
            } else {
                return Err(MessageHandleError::InvalidMessageFormat);
            }
        }

        // ["COUNT", <subscription_id>, <filter JSON>]
        if v[0] == "COUNT" {
            if v_len >= 3 {
                let subscription_id: SubscriptionId = serde_json::from_value(v[1].clone())?;
                let filter: Filter = serde_json::from_value(v[2].clone())?;
                return Ok(Self::count(subscription_id, filter));
            } else {
                return Err(MessageHandleError::InvalidMessageFormat);
            }
        }

        // Close
        // ["CLOSE", <subscription_id>]
        if v[0] == "CLOSE" {
            if v_len >= 2 {
                let subscription_id: SubscriptionId = serde_json::from_value(v[1].clone())?;
                return Ok(Self::close(subscription_id));
            } else {
                return Err(MessageHandleError::InvalidMessageFormat);
            }
        }

        // Auth
        // ["AUTH", <event JSON>]
        if v[0] == "AUTH" {
            if v_len >= 2 {
                let event: Event = serde_json::from_value(v[1].clone())?;
                return Ok(Self::auth(event));
            } else {
                return Err(MessageHandleError::InvalidMessageFormat);
            }
        }

        // Negentropy Open
        // New: ["NEG-OPEN", <subscription ID string>, <filter>, <initialMessage as lowercase hex-encoded>]
        // Old: ["NEG-OPEN", <subscription ID string>, <filter>, <idSize>, <initialMessage as lowercase hex-encoded>]
        if v[0] == "NEG-OPEN" {
            // New negentropy protocol message
            if v_len == 4 {
                let subscription_id: SubscriptionId = serde_json::from_value(v[1].clone())?;
                let filter: Filter = Filter::from_json(v[2].to_string())?;
                let initial_message: String = serde_json::from_value(v[3].clone())?;
                return Ok(Self::NegOpen {
                    subscription_id,
                    filter: Box::new(filter),
                    id_size: None,
                    initial_message,
                });
            }

            // Old negentropy protocol message
            if v_len == 5 {
                let subscription_id: SubscriptionId = serde_json::from_value(v[1].clone())?;
                let filter: Filter = Filter::from_json(v[2].to_string())?;
                let id_size: u8 =
                    v[3].as_u64()
                        .ok_or(MessageHandleError::InvalidMessageFormat)? as u8;
                let initial_message: String = serde_json::from_value(v[4].clone())?;
                return Ok(Self::NegOpen {
                    subscription_id,
                    filter: Box::new(filter),
                    id_size: Some(id_size),
                    initial_message,
                });
            }

            return Err(MessageHandleError::InvalidMessageFormat);
        }

        // Negentropy Message
        // ["NEG-MSG", <subscription ID string>, <message, lowercase hex-encoded>]
        if v[0] == "NEG-MSG" {
            if v_len >= 3 {
                let subscription_id: SubscriptionId = serde_json::from_value(v[1].clone())?;
                let message: String = serde_json::from_value(v[2].clone())?;
                return Ok(Self::NegMsg {
                    subscription_id,
                    message,
                });
            } else {
                return Err(MessageHandleError::InvalidMessageFormat);
            }
        }

        // Negentropy Close
        // ["NEG-CLOSE", <subscription ID string>]
        if v[0] == "NEG-CLOSE" {
            if v_len >= 2 {
                let subscription_id: SubscriptionId = serde_json::from_value(v[1].clone())?;
                return Ok(Self::NegClose { subscription_id });
            } else {
                return Err(MessageHandleError::InvalidMessageFormat);
            }
        }

        Err(MessageHandleError::InvalidMessageFormat)
    }
}

impl JsonUtil for ClientMessage {
    type Err = MessageHandleError;

    /// Deserialize [`ClientMessage`] from JSON string
    ///
    /// **This method doesn't verify the event signature!**
    fn from_json<T>(json: T) -> Result<Self, Self::Err>
    where
        T: AsRef<[u8]>,
    {
        let msg: &[u8] = json.as_ref();

        if msg.is_empty() {
            return Err(MessageHandleError::InvalidMessageFormat);
        }

        let value: Value = serde_json::from_slice(msg)?;
        Self::from_value(value)
    }
}

#[cfg(test)]
mod tests {
    use core::str::FromStr;

    use super::*;
    use crate::{Kind, PublicKey};

    #[test]
    fn test_client_message_req() {
        let pk =
            PublicKey::from_str("379e863e8357163b5bce5d2688dc4f1dcc2d505222fb8d74db600f30535dfdfe")
                .unwrap();

        let client_req = ClientMessage::req(SubscriptionId::new("test"), Filter::new().pubkey(pk));
        assert_eq!(
            client_req.as_json(),
            r##"["REQ","test",{"#p":["379e863e8357163b5bce5d2688dc4f1dcc2d505222fb8d74db600f30535dfdfe"]}]"##
        );
    }

    #[test]
    fn test_client_message_custom_kind() {
        let client_req = ClientMessage::req(
            SubscriptionId::new("test"),
            Filter::new().kind(Kind::Custom(22)),
        );
        assert_eq!(client_req.as_json(), r##"["REQ","test",{"kinds":[22]}]"##);
    }
}
