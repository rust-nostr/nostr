// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Raw Relay messages

use alloc::string::String;
use serde_json::Value;

use crate::message::MessageHandleError;

/// Raw Relay Message
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RawRelayMessage {
    /// `["EVENT", <subscription_id>, <event JSON>]` (NIP01)
    Event {
        /// Subscription ID
        subscription_id: String,
        /// Event JSON
        event: Value,
    },
    /// `["OK", <event_id>, <true|false>, <message>]` (NIP01)
    Ok {
        /// Event ID
        event_id: String,
        /// Status
        status: bool,
        /// Message
        message: String,
    },
    /// `["EOSE", <subscription_id>]` (NIP01)
    EndOfStoredEvents(String),
    /// ["NOTICE", \<message\>] (NIP01)
    Notice {
        /// Message
        message: String,
    },
    /// `["AUTH", <challenge-string>]` (NIP42)
    Auth {
        /// Challenge
        challenge: String,
    },
    /// `["COUNT", <subscription_id>, {"count": <integer>}]` (NIP45)
    Count {
        /// Subscription ID
        subscription_id: String,
        /// Events count
        count: usize,
    },
    /// Negentropy Message
    NegMsg {
        /// Subscription ID
        subscription_id: String,
        /// Message
        message: String,
    },
    /// Negentropy Error
    NegErr {
        /// Subscription ID
        subscription_id: String,
        /// Error code
        code: String,
    },
}

impl RawRelayMessage {
    /// Deserialize [`RawRelayMessage`] from [`Value`]
    pub fn from_value(msg: Value) -> Result<Self, MessageHandleError> {
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
            return Ok(Self::Notice {
                message: serde_json::from_value(v[1].clone())?,
            });
        }

        // Event
        // Relay response format: ["EVENT", <subscription id>, <event JSON>]
        if v[0] == "EVENT" {
            if v_len != 3 {
                return Err(MessageHandleError::InvalidMessageFormat);
            }

            return Ok(Self::Event {
                subscription_id: serde_json::from_value(v[1].clone())?,
                event: v[2].clone(),
            });
        }

        // EOSE (NIP-15)
        // Relay response format: ["EOSE", <subscription_id>]
        if v[0] == "EOSE" {
            if v_len != 2 {
                return Err(MessageHandleError::InvalidMessageFormat);
            }

            let subscription_id: String = serde_json::from_value(v[1].clone())?;
            return Ok(Self::EndOfStoredEvents(subscription_id));
        }

        // OK (NIP-20)
        // Relay response format: ["OK", <event_id>, <true|false>, <message>]
        if v[0] == "OK" {
            if v_len != 4 {
                return Err(MessageHandleError::InvalidMessageFormat);
            }

            return Ok(Self::Ok {
                event_id: serde_json::from_value(v[1].clone())?,
                status: serde_json::from_value(v[2].clone())?,
                message: serde_json::from_value(v[3].clone())?,
            });
        }

        // OK (NIP-42)
        // Relay response format: ["AUTH", <challenge>]
        if v[0] == "AUTH" {
            if v_len != 2 {
                return Err(MessageHandleError::InvalidMessageFormat);
            }

            return Ok(Self::Auth {
                challenge: serde_json::from_value(v[1].clone())?,
            });
        }

        // Relay response format: ["EVENT", <subscription id>, <event JSON>]
        if v[0] == "COUNT" {
            if v_len != 3 {
                return Err(MessageHandleError::InvalidMessageFormat);
            }

            let map = v[2]
                .as_object()
                .ok_or(MessageHandleError::InvalidMessageFormat)?;
            let count: Value = map
                .get("count")
                .ok_or(MessageHandleError::InvalidMessageFormat)?
                .clone();
            let count: usize = serde_json::from_value(count)?;

            return Ok(Self::Count {
                subscription_id: serde_json::from_value(v[1].clone())?,
                count,
            });
        }

        // Negentropy Message
        // ["NEG-MSG", <subscription ID string>, <message, lowercase hex-encoded>]
        if v[0] == "NEG-MSG" {
            if v_len != 3 {
                return Err(MessageHandleError::InvalidMessageFormat);
            }

            return Ok(Self::NegMsg {
                subscription_id: serde_json::from_value(v[1].clone())?,
                message: serde_json::from_value(v[2].clone())?,
            });
        }

        // Negentropy Error
        // ["NEG-ERR", <subscription ID string>, <reason-code>]
        if v[0] == "NEG-ERR" {
            if v_len != 3 {
                return Err(MessageHandleError::InvalidMessageFormat);
            }

            return Ok(Self::NegErr {
                subscription_id: serde_json::from_value(v[1].clone())?,
                code: serde_json::from_value(v[2].clone())?,
            });
        }

        Err(MessageHandleError::InvalidMessageFormat)
    }

    /// Deserialize [`RawRelayMessage`] from JSON string
    pub fn from_json<T>(json: T) -> Result<Self, MessageHandleError>
    where
        T: AsRef<[u8]>,
    {
        let msg: &[u8] = json.as_ref();

        if msg.is_empty() {
            return Err(MessageHandleError::EmptyMsg);
        }

        let value: Value = serde_json::from_slice(msg)?;
        Self::from_value(value)
    }
}
