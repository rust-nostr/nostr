// Copyright (c) 2021 Paul Miller
// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Relay messages

use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::IntoIter;
use core::fmt;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{json, Value};

use super::MessageHandleError;
use crate::{Event, EventId, JsonUtil, SubscriptionId};

/// Machine-readable prefixes for `OK` and `CLOSED` relay messages
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MachineReadablePrefix {
    /// Duplicate
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    Duplicate,
    /// POW
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    Pow,
    /// Blocked
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    Blocked,
    /// Rate limited
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    RateLimited,
    /// Invalid
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    Invalid,
    /// Error
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    Error,
    /// Unsupported
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    Unsupported,
    /// Authentication required
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/42.md>
    AuthRequired,
    /// Restricted
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/42.md>
    Restricted,
}

impl fmt::Display for MachineReadablePrefix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Duplicate => write!(f, "duplicate"),
            Self::Pow => write!(f, "pow"),
            Self::Blocked => write!(f, "blocked"),
            Self::RateLimited => write!(f, "rate-limited"),
            Self::Invalid => write!(f, "invalid"),
            Self::Error => write!(f, "error"),
            Self::Unsupported => write!(f, "unsupported"),
            Self::AuthRequired => write!(f, "auth-required"),
            Self::Restricted => write!(f, "restricted"),
        }
    }
}

impl MachineReadablePrefix {
    /// Parse machine-readable prefix
    pub fn parse(message: &str) -> Option<Self> {
        match message {
            m if m.starts_with("duplicate:") => Some(Self::Duplicate),
            m if m.starts_with("pow:") => Some(Self::Pow),
            m if m.starts_with("blocked:") => Some(Self::Blocked),
            m if m.starts_with("rate-limited:") => Some(Self::RateLimited),
            m if m.starts_with("invalid:") => Some(Self::Invalid),
            m if m.starts_with("error:") => Some(Self::Error),
            m if m.starts_with("unsupported:") => Some(Self::Unsupported),
            m if m.starts_with("auth-required:") => Some(Self::AuthRequired),
            m if m.starts_with("restricted:") => Some(Self::Restricted),
            _ => None,
        }
    }
}

/// Messages sent by relays, received by clients
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RelayMessage<'a> {
    /// Event
    ///
    /// Used to send events requested by clients.
    ///
    /// JSON: `["EVENT", <subscription_id>, <event JSON>]`.
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    Event {
        /// Subscription ID
        subscription_id: Cow<'a, SubscriptionId>,
        /// Event
        event: Cow<'a, Event>,
    },
    /// Ok
    ///
    /// Used to indicate acceptance or denial of an `EVENT` message.
    ///
    /// JSON: `["OK", <event_id>, <true|false>, <message>]`.
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    Ok {
        /// Event ID
        event_id: EventId,
        /// Status
        status: bool,
        /// Message
        message: Cow<'a, str>,
    },
    /// End of stored events
    ///
    /// Used to indicate the end of stored events and the beginning of events newly received in real-time.
    ///
    /// JSON: `["EOSE", <subscription_id>]`.
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    EndOfStoredEvents(Cow<'a, SubscriptionId>),
    /// Notice
    ///
    /// Used to send human-readable error messages or other things to clients.
    ///
    /// JSON: `["NOTICE", <message>]`.
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    Notice(Cow<'a, str>),
    /// Closed
    ///
    /// Used to indicate that a subscription was ended on the server side.
    ///
    /// JSON: `["CLOSED", <subscription_id>, <message>]`.
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    Closed {
        /// Subscription ID
        subscription_id: Cow<'a, SubscriptionId>,
        /// Message
        message: Cow<'a, str>,
    },
    /// Auth
    ///
    /// `["AUTH", <challenge-string>]`
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/42.md>
    Auth {
        /// Challenge
        challenge: Cow<'a, str>,
    },
    /// Count
    ///
    /// `["COUNT", <subscription_id>, {"count": <integer>}]`
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/45.md>
    Count {
        /// Subscription ID
        subscription_id: Cow<'a, SubscriptionId>,
        /// Events count
        count: usize,
    },
    /// Negentropy Message
    NegMsg {
        /// Subscription ID
        subscription_id: Cow<'a, SubscriptionId>,
        /// Message
        message: Cow<'a, str>,
    },
    /// Negentropy Error
    NegErr {
        /// Subscription ID
        subscription_id: Cow<'a, SubscriptionId>,
        /// Error message
        message: Cow<'a, str>,
    },
}

impl RelayMessage<'_> {
    /// Create `EVENT` message
    #[inline]
    pub fn event(subscription_id: SubscriptionId, event: Event) -> Self {
        Self::Event {
            subscription_id: Cow::Owned(subscription_id),
            event: Cow::Owned(event),
        }
    }

    /// Create `NOTICE` message
    #[inline]
    pub fn notice<S>(message: S) -> Self
    where
        S: Into<String>,
    {
        Self::Notice(Cow::Owned(message.into()))
    }

    /// Create `CLOSED` message
    #[inline]
    pub fn closed<S>(subscription_id: SubscriptionId, message: S) -> Self
    where
        S: Into<String>,
    {
        Self::Closed {
            subscription_id: Cow::Owned(subscription_id),
            message: Cow::Owned(message.into()),
        }
    }

    /// Create `EOSE` message
    #[inline]
    pub fn eose(subscription_id: SubscriptionId) -> Self {
        Self::EndOfStoredEvents(Cow::Owned(subscription_id))
    }

    /// Create `OK` message
    #[inline]
    pub fn ok<S>(event_id: EventId, status: bool, message: S) -> Self
    where
        S: Into<String>,
    {
        Self::Ok {
            event_id,
            status,
            message: Cow::Owned(message.into()),
        }
    }

    /// Create `AUTH` message
    #[inline]
    pub fn auth<S>(challenge: S) -> Self
    where
        S: Into<String>,
    {
        Self::Auth {
            challenge: Cow::Owned(challenge.into()),
        }
    }

    /// Create  `EVENT` message
    #[inline]
    pub fn count(subscription_id: SubscriptionId, count: usize) -> Self {
        Self::Count {
            subscription_id: Cow::Owned(subscription_id),
            count,
        }
    }

    /// Deserialize from [`Value`]
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
                let message: String = next_and_deser(&mut v_iter)?; // Index 1
                Ok(Self::notice(message))
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
                let subscription_id: SubscriptionId = next_and_deser(&mut v_iter)?; // Index 1
                Ok(Self::eose(subscription_id))
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
                let subscription_id: SubscriptionId = next_and_deser(&mut v_iter)?; // Index 1
                let Count { count } = next_and_deser(&mut v_iter)?; // Index 2

                Ok(Self::Count {
                    subscription_id: Cow::Owned(subscription_id),
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
                    message: next_and_deser(&mut v_iter)?,         // Index 2
                })
            }
            _ => Err(MessageHandleError::InvalidMessageFormat),
        }
    }

    fn as_value(&self) -> Value {
        match self {
            Self::Event {
                event,
                subscription_id,
            } => json!(["EVENT", subscription_id, event]),
            Self::Notice(message) => json!(["NOTICE", message]),
            Self::Closed {
                subscription_id,
                message,
            } => json!(["CLOSED", subscription_id, message]),
            Self::EndOfStoredEvents(subscription_id) => {
                json!(["EOSE", subscription_id])
            }
            Self::Ok {
                event_id,
                status,
                message,
            } => json!(["OK", event_id, status, message]),
            Self::Auth { challenge } => json!(["AUTH", challenge]),
            Self::Count {
                subscription_id,
                count,
            } => json!(["COUNT", subscription_id, { "count": count }]),
            Self::NegMsg {
                subscription_id,
                message,
            } => json!(["NEG-MSG", subscription_id, message]),
            Self::NegErr {
                subscription_id,
                message,
            } => json!(["NEG-ERR", subscription_id, message]),
        }
    }
}

impl Serialize for RelayMessage<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let json_value: Value = self.as_value();
        json_value.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for RelayMessage<'_> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let json_value = Value::deserialize(deserializer)?;
        RelayMessage::from_value(json_value).map_err(serde::de::Error::custom)
    }
}

impl JsonUtil for RelayMessage<'_> {
    type Err = MessageHandleError;

    /// Deserialize [`RelayMessage`] from JSON string
    ///
    /// **This method doesn't verify the event signature!**
    fn from_json<T>(json: T) -> Result<Self, Self::Err>
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

#[cfg(feature = "std")]
#[cfg(test)]
mod tests {
    use core::str::FromStr;

    use secp256k1::schnorr::Signature;

    use super::*;
    use crate::{Kind, PublicKey, Timestamp};

    #[test]
    fn test_handle_valid_notice() {
        let valid_notice_msg = r#"["NOTICE","Invalid event format!"]"#;
        let handled_valid_notice_msg = RelayMessage::notice(String::from("Invalid event format!"));

        assert_eq!(
            RelayMessage::from_json(valid_notice_msg).unwrap(),
            handled_valid_notice_msg
        );
    }
    #[test]
    fn test_handle_invalid_notice() {
        // Missing content
        let invalid_notice_msg = r#"["NOTICE"]"#;
        // The content is not string
        let invalid_notice_msg_content = r#"["NOTICE": 404]"#;

        assert!(RelayMessage::from_json(invalid_notice_msg).is_err(),);
        assert!(RelayMessage::from_json(invalid_notice_msg_content).is_err(),);
    }

    #[test]
    fn test_handle_valid_closed() {
        let valid_closed_msg = r#"["CLOSED","random-subscription-id","reason"]"#;
        let handled_valid_closed_msg =
            RelayMessage::closed(SubscriptionId::new("random-subscription-id"), "reason");

        assert_eq!(
            RelayMessage::from_json(valid_closed_msg).unwrap(),
            handled_valid_closed_msg
        );
    }

    #[test]
    fn test_handle_invalid_closed() {
        // Missing subscription ID
        assert!(RelayMessage::from_json(r#"["CLOSED"]"#).is_err());

        // The subscription ID is not a string
        assert!(RelayMessage::from_json(r#"["CLOSED", 404, "reason"]"#).is_err());

        // The content is not a string
        assert!(RelayMessage::from_json(r#"["CLOSED", "random-subscription-id", 404]"#).is_err())
    }

    #[test]
    fn test_handle_valid_event() {
        let valid_event_msg = r#"["EVENT", "random_string", {"id":"70b10f70c1318967eddf12527799411b1a9780ad9c43858f5e5fcd45486a13a5","pubkey":"379e863e8357163b5bce5d2688dc4f1dcc2d505222fb8d74db600f30535dfdfe","created_at":1612809991,"kind":1,"tags":[],"content":"test","sig":"273a9cd5d11455590f4359500bccb7a89428262b96b3ea87a756b770964472f8c3e87f5d5e64d8d2e859a71462a3f477b554565c4f2f326cb01dd7620db71502"}]"#;

        let id =
            EventId::from_hex("70b10f70c1318967eddf12527799411b1a9780ad9c43858f5e5fcd45486a13a5")
                .unwrap();
        let pubkey =
            PublicKey::from_str("379e863e8357163b5bce5d2688dc4f1dcc2d505222fb8d74db600f30535dfdfe")
                .unwrap();
        let created_at = Timestamp::from(1612809991);
        let kind = Kind::TextNote;
        let content = "test";
        let sig = Signature::from_str("273a9cd5d11455590f4359500bccb7a89428262b96b3ea87a756b770964472f8c3e87f5d5e64d8d2e859a71462a3f477b554565c4f2f326cb01dd7620db71502").unwrap();

        let handled_event = Event::new(id, pubkey, created_at, kind, [], content, sig);

        assert_eq!(
            RelayMessage::from_json(valid_event_msg).unwrap(),
            RelayMessage::event(SubscriptionId::new("random_string"), handled_event)
        );

        let message = RelayMessage::from_json(r#"["EVENT","bf7da933d6c6d67e5c97f94f17cf8762",{"content":"Think about this.\n\nThe most powerful centralized institutions in the world have been replaced by a protocol that protects the individual. #bitcoin\n\nDo you doubt that we can replace everything else?\n\nBullish on the future of humanity\nnostr:nevent1qqs9ljegkuk2m2ewfjlhxy054n6ld5dfngwzuep0ddhs64gc49q0nmqpzdmhxue69uhhyetvv9ukzcnvv5hx7un8qgsw3mfhnrr0l6ll5zzsrtpeufckv2lazc8k3ru5c3wkjtv8vlwngksrqsqqqqqpttgr27","created_at":1703184271,"id":"38acf9b08d06859e49237688a9fd6558c448766f47457236c2331f93538992c6","kind":1,"pubkey":"e8ed3798c6ffebffa08501ac39e271662bfd160f688f94c45d692d8767dd345a","sig":"f76d5ecc8e7de688ac12b9d19edaacdcffb8f0c8fa2a44c00767363af3f04dbc069542ddc5d2f63c94cb5e6ce701589d538cf2db3b1f1211a96596fabb6ecafe","tags":[["e","5fcb28b72cadab2e4cbf7311f4acf5f6d1a99a1c2e642f6b6f0d5518a940f9ec","","mention"],["p","e8ed3798c6ffebffa08501ac39e271662bfd160f688f94c45d692d8767dd345a","","mention"],["t","bitcoin"],["t","bitcoin"]]}]"#).unwrap();
        if let RelayMessage::Event { event, .. } = message {
            event.verify().unwrap();
        } else {
            panic!("Wrong relay message");
        }
    }

    #[test]
    fn test_handle_invalid_event() {
        // Missing Event field
        let invalid_event_msg = r#"["EVENT", "random_string"]"#;
        // Event JSON with incomplete content
        let invalid_event_msg_content = r#"["EVENT", "random_string", {"id":"70b10f70c1318967eddf12527799411b1a9780ad9c43858f5e5fcd45486a13a5","pubkey":"379e863e8357163b5bce5d2688dc4f1dcc2d505222fb8d74db600f30535dfdfe"}]"#;

        assert!(RelayMessage::from_json(invalid_event_msg).is_err());

        assert!(RelayMessage::from_json(invalid_event_msg_content).is_err());
    }

    #[test]
    fn test_handle_valid_eose() {
        let valid_eose_msg = r#"["EOSE","random-subscription-id"]"#;
        let handled_valid_eose_msg =
            RelayMessage::eose(SubscriptionId::new("random-subscription-id"));

        assert_eq!(
            RelayMessage::from_json(valid_eose_msg).unwrap(),
            handled_valid_eose_msg
        );
    }
    #[test]
    fn test_handle_invalid_eose() {
        // Missing subscription ID
        assert!(RelayMessage::from_json(r#"["EOSE"]"#).is_err(),);

        // The subscription ID is not string
        assert!(RelayMessage::from_json(r#"["EOSE", 404]"#).is_err(),);
    }

    #[test]
    fn test_handle_valid_ok() {
        let valid_ok_msg = r#"["OK", "b1a649ebe8b435ec71d3784793f3bbf4b93e64e17568a741aecd4c7ddeafce30", true, "pow: difficulty 25>=24"]"#;
        let handled_valid_ok_msg = RelayMessage::ok(
            EventId::from_hex("b1a649ebe8b435ec71d3784793f3bbf4b93e64e17568a741aecd4c7ddeafce30")
                .unwrap(),
            true,
            "pow: difficulty 25>=24",
        );

        assert_eq!(
            RelayMessage::from_json(valid_ok_msg).unwrap(),
            handled_valid_ok_msg
        );
    }
    #[test]
    fn test_handle_invalid_ok() {
        // Missing params
        assert!(RelayMessage::from_json(
            r#"["OK", "b1a649ebe8b435ec71d3784793f3bbf4b93e64e17568a741aecd4c7ddeafce30"]"#
        )
        .is_err());

        // Invalid event_id
        assert!(RelayMessage::from_json(
            r#"["OK", "b1a649ebe8b435ec71d3784793f3bbf4b93e64e17568a741aecd4c7dde", true, ""]"#
        )
        .is_err());

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
    fn parse_message() {
        // Got this fresh off the wire
        pub const SAMPLE_EVENT: &str = r#"["EVENT", "random_string", {"id":"70b10f70c1318967eddf12527799411b1a9780ad9c43858f5e5fcd45486a13a5","pubkey":"379e863e8357163b5bce5d2688dc4f1dcc2d505222fb8d74db600f30535dfdfe","created_at":1612809991,"kind":1,"tags":[],"content":"test","sig":"273a9cd5d11455590f4359500bccb7a89428262b96b3ea87a756b770964472f8c3e87f5d5e64d8d2e859a71462a3f477b554565c4f2f326cb01dd7620db71502"}]"#;

        // Hand parsed version as a sanity check
        let id =
            EventId::from_hex("70b10f70c1318967eddf12527799411b1a9780ad9c43858f5e5fcd45486a13a5")
                .unwrap();
        let pubkey =
            PublicKey::from_str("379e863e8357163b5bce5d2688dc4f1dcc2d505222fb8d74db600f30535dfdfe")
                .unwrap();
        let created_at = Timestamp::from(1612809991);
        let kind = Kind::TextNote;
        let content = "test";
        let sig = Signature::from_str("273a9cd5d11455590f4359500bccb7a89428262b96b3ea87a756b770964472f8c3e87f5d5e64d8d2e859a71462a3f477b554565c4f2f326cb01dd7620db71502").unwrap();

        let event = Event::new(id, pubkey, created_at, kind, [], content, sig);

        let parsed_event = RelayMessage::from_json(SAMPLE_EVENT).expect("Failed to parse event");

        assert_eq!(
            parsed_event,
            RelayMessage::event(SubscriptionId::new("random_string"), event)
        );
    }
}

#[cfg(bench)]
mod benches {
    use test::{black_box, Bencher};

    use super::*;

    #[bench]
    pub fn parse_ok_relay_message(bh: &mut Bencher) {
        let json: &str = r#"["OK", "70b10f70c1318967eddf12527799411b1a9780ad9c43858f5e5fcd45486a13a5", true, "pow: difficulty 25>=24"]"#;
        bh.iter(|| {
            black_box(RelayMessage::from_json(&json)).unwrap();
        });
    }

    #[bench]
    pub fn parse_event_relay_message(bh: &mut Bencher) {
        let json: &str = r#"["EVENT", "random_string", {"id":"70b10f70c1318967eddf12527799411b1a9780ad9c43858f5e5fcd45486a13a5","pubkey":"379e863e8357163b5bce5d2688dc4f1dcc2d505222fb8d74db600f30535dfdfe","created_at":1612809991,"kind":1,"tags":[],"content":"test","sig":"273a9cd5d11455590f4359500bccb7a89428262b96b3ea87a756b770964472f8c3e87f5d5e64d8d2e859a71462a3f477b554565c4f2f326cb01dd7620db71502"}]"#;
        bh.iter(|| {
            black_box(RelayMessage::from_json(&json)).unwrap();
        });
    }
}
