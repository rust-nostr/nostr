// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Raw Relay messages

use alloc::string::String;

use serde_json::Value;

use crate::event::raw::RawEvent;
use crate::message::MessageHandleError;

/// Raw Relay Message
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RawRelayMessage {
    /// `["EVENT", <subscription_id>, <event JSON>]` (NIP01)
    Event {
        /// Subscription ID
        subscription_id: String,
        /// Event JSON
        event: RawEvent,
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
    /// `["NOTICE", <message>]` (NIP01)
    Notice {
        /// Message
        message: String,
    },
    /// `["CLOSED", <subscription_id>, <message>]` (NIP01)
    Closed {
        /// Subscription ID
        subscription_id: String,
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
        let v_type: &str = v[0]
            .as_str()
            .ok_or(MessageHandleError::InvalidMessageFormat)?;

        // Notice
        // Relay response format: ["NOTICE", <message>]
        if v_type == "NOTICE" {
            return if v_len >= 2 {
                Ok(Self::Notice {
                    message: serde_json::from_value(v[1].clone())?,
                })
            } else {
                Err(MessageHandleError::InvalidMessageFormat)
            };
        }

        // Closed
        // Relay response format: ["CLOSED", <subscription_id>, <message>]
        if v_type == "CLOSED" {
            return if v_len >= 3 {
                Ok(Self::Closed {
                    subscription_id: serde_json::from_value(v[1].clone())?,
                    message: serde_json::from_value(v[2].clone())?,
                })
            } else {
                Err(MessageHandleError::InvalidMessageFormat)
            };
        }

        // Event
        // Relay response format: ["EVENT", <subscription id>, <event JSON>]
        if v_type == "EVENT" {
            return if v_len >= 3 {
                Ok(Self::Event {
                    subscription_id: serde_json::from_value(v[1].clone())?,
                    event: serde_json::from_value(v[2].clone())?,
                })
            } else {
                Err(MessageHandleError::InvalidMessageFormat)
            };
        }

        // EOSE (NIP-15)
        // Relay response format: ["EOSE", <subscription_id>]
        if v_type == "EOSE" {
            return if v_len >= 2 {
                let subscription_id: String = serde_json::from_value(v[1].clone())?;
                Ok(Self::EndOfStoredEvents(subscription_id))
            } else {
                Err(MessageHandleError::InvalidMessageFormat)
            };
        }

        // OK (NIP-20)
        // Relay response format: ["OK", <event_id>, <true|false>, <message>]
        if v_type == "OK" {
            return if v_len >= 4 {
                Ok(Self::Ok {
                    event_id: serde_json::from_value(v[1].clone())?,
                    status: serde_json::from_value(v[2].clone())?,
                    message: serde_json::from_value(v[3].clone())?,
                })
            } else {
                Err(MessageHandleError::InvalidMessageFormat)
            };
        }

        // OK (NIP-42)
        // Relay response format: ["AUTH", <challenge>]
        if v_type == "AUTH" {
            return if v_len >= 2 {
                Ok(Self::Auth {
                    challenge: serde_json::from_value(v[1].clone())?,
                })
            } else {
                Err(MessageHandleError::InvalidMessageFormat)
            };
        }

        // Relay response format: ["COUNT", <subscription id>, {"count": num}]
        if v_type == "COUNT" {
            return if v_len >= 3 {
                let map = v[2]
                    .as_object()
                    .ok_or(MessageHandleError::InvalidMessageFormat)?;
                let count: Value = map
                    .get("count")
                    .ok_or(MessageHandleError::InvalidMessageFormat)?
                    .clone();
                let count: usize = serde_json::from_value(count)?;

                Ok(Self::Count {
                    subscription_id: serde_json::from_value(v[1].clone())?,
                    count,
                })
            } else {
                Err(MessageHandleError::InvalidMessageFormat)
            };
        }

        // Negentropy Message
        // ["NEG-MSG", <subscription ID string>, <message, lowercase hex-encoded>]
        if v_type == "NEG-MSG" {
            return if v_len >= 3 {
                Ok(Self::NegMsg {
                    subscription_id: serde_json::from_value(v[1].clone())?,
                    message: serde_json::from_value(v[2].clone())?,
                })
            } else {
                Err(MessageHandleError::InvalidMessageFormat)
            };
        }

        // Negentropy Error
        // ["NEG-ERR", <subscription ID string>, <reason-code>]
        if v_type == "NEG-ERR" {
            return if v_len >= 3 {
                Ok(Self::NegErr {
                    subscription_id: serde_json::from_value(v[1].clone())?,
                    code: serde_json::from_value(v[2].clone())?,
                })
            } else {
                Err(MessageHandleError::InvalidMessageFormat)
            };
        }

        Err(MessageHandleError::InvalidMessageFormat)
    }

    /// Deserialize [`RawRelayMessage`] from JSON string
    #[inline]
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

#[cfg(bench)]
mod benches {
    use test::{black_box, Bencher};

    use super::*;

    const EVENT: &'static str = r#"["EVENT", "random_string", {"id":"70b10f70c1318967eddf12527799411b1a9780ad9c43858f5e5fcd45486a13a5","pubkey":"379e863e8357163b5bce5d2688dc4f1dcc2d505222fb8d74db600f30535dfdfe","created_at":1612809991,"kind":1,"tags":[],"content":"test","sig":"273a9cd5d11455590f4359500bccb7a89428262b96b3ea87a756b770964472f8c3e87f5d5e64d8d2e859a71462a3f477b554565c4f2f326cb01dd7620db71502"}]"#;

    #[bench]
    pub fn deserialize_raw_message_relay(bh: &mut Bencher) {
        bh.iter(|| {
            black_box(RawRelayMessage::from_json(EVENT)).unwrap();
        });
    }
}
