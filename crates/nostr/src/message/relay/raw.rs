// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Raw Relay messages

use alloc::string::String;
use alloc::vec::IntoIter;

use serde::de::DeserializeOwned;
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
        let Value::Array(v) = msg else {
            return Err(MessageHandleError::InvalidMessageFormat);
        };

        if v.is_empty() {
            return Err(MessageHandleError::InvalidMessageFormat);
        }

        let mut v_iter = v.into_iter();

        // Index 0
        let v_type: String = next_and_deser(&mut v_iter)?;

        match v_type.as_str() {
            "NOTICE" => {
                // ["NOTICE", <message>]
                Ok(Self::Notice {
                    message: next_and_deser(&mut v_iter)?, // Index 1
                })
            }
            "CLOSED" => {
                // ["CLOSED", <subscription_id>, <message>]
                Ok(Self::Closed {
                    subscription_id: next_and_deser(&mut v_iter)?, // Index 1
                    message: next_and_deser(&mut v_iter)?,         // Index 2
                })
            }
            "EVENT" => {
                // ["EVENT", <subscription id>, <event JSON>]
                Ok(Self::Event {
                    subscription_id: next_and_deser(&mut v_iter)?, // Index 1
                    event: next_and_deser(&mut v_iter)?,           // Index 2
                })
            }
            "EOSE" => {
                // ["EOSE", <subscription_id>]
                let subscription_id: String = next_and_deser(&mut v_iter)?; // Index 1
                Ok(Self::EndOfStoredEvents(subscription_id))
            }
            "OK" => {
                // ["OK", <event_id>, <true|false>, <message>]
                Ok(Self::Ok {
                    event_id: next_and_deser(&mut v_iter)?, // Index 1
                    status: next_and_deser(&mut v_iter)?,   // Index 2
                    message: next_and_deser(&mut v_iter)?,  // Index 3
                })
            }
            "AUTH" => {
                // ["AUTH", <challenge>]
                Ok(Self::Auth {
                    challenge: next_and_deser(&mut v_iter)?, // Index 1
                })
            }
            "COUNT" => {
                // ["COUNT", <subscription id>, {"count": num}]
                let subscription_id: String = next_and_deser(&mut v_iter)?; // Index 1
                let Count { count } = next_and_deser(&mut v_iter)?; // Index 2

                Ok(Self::Count {
                    subscription_id,
                    count,
                })
            }
            "NEG-MSG" => {
                // ["NEG-MSG", <subscription ID string>, <message, lowercase hex-encoded>]
                Ok(Self::NegMsg {
                    subscription_id: next_and_deser(&mut v_iter)?, // Index 1
                    message: next_and_deser(&mut v_iter)?,         // Index 2
                })
            }
            "NEG-ERR" => {
                // ["NEG-ERR", <subscription ID string>, <reason-code>]
                Ok(Self::NegErr {
                    subscription_id: next_and_deser(&mut v_iter)?, // Index 1
                    code: next_and_deser(&mut v_iter)?,            // Index 2
                })
            }
            _ => Err(MessageHandleError::InvalidMessageFormat),
        }
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

#[inline]
fn next_and_deser<T>(iter: &mut IntoIter<Value>) -> Result<T, MessageHandleError>
where
    T: DeserializeOwned,
{
    let val: Value = iter
        .next()
        .ok_or(MessageHandleError::InvalidMessageFormat)?;
    Ok(serde_json::from_value(val)?)
}

#[derive(Deserialize)]
struct Count {
    count: usize,
}

#[cfg(bench)]
mod benches {
    use test::{black_box, Bencher};

    use super::*;

    const EVENT: &'static str = r#"["EVENT", "random_string", {"id":"70b10f70c1318967eddf12527799411b1a9780ad9c43858f5e5fcd45486a13a5","pubkey":"379e863e8357163b5bce5d2688dc4f1dcc2d505222fb8d74db600f30535dfdfe","created_at":1612809991,"kind":1,"tags":[],"content":"test","sig":"273a9cd5d11455590f4359500bccb7a89428262b96b3ea87a756b770964472f8c3e87f5d5e64d8d2e859a71462a3f477b554565c4f2f326cb01dd7620db71502"}]"#;

    #[bench]
    pub fn parse_raw_message_relay(bh: &mut Bencher) {
        bh.iter(|| {
            black_box(RawRelayMessage::from_json(EVENT)).unwrap();
        });
    }
}
