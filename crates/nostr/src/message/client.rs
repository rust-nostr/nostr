// Copyright (c) 2021 Paul Miller
// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use serde_json::{json, Value};

use super::MessageHandleError;
use crate::{Event, SubscriptionFilter};

/// Messages sent by clients, received by relays
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ClientMessage {
    Event {
        event: Event,
    },
    Req {
        subscription_id: String,
        filters: Vec<SubscriptionFilter>,
    },
    Close {
        subscription_id: String,
    },
}

impl ClientMessage {
    pub fn new_event(event: Event) -> Self {
        Self::Event { event }
    }

    pub fn new_req(subscription_id: impl Into<String>, filters: Vec<SubscriptionFilter>) -> Self {
        Self::Req {
            subscription_id: subscription_id.into(),
            filters,
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
        }
    }

    pub fn from_json(msg: &str) -> Result<Self, MessageHandleError> {
        log::trace!("{}", msg);

        let v: Vec<Value> =
            serde_json::from_str(msg).map_err(|_| MessageHandleError::JsonDeserializationFailed)?;

        if v.is_empty() {
            return Err(MessageHandleError::InvalidMessageFormat);
        }

        // Event
        // ["EVENT", <event JSON>]
        if v[0] == "EVENT" {
            if v.len() != 2 {
                return Err(MessageHandleError::InvalidMessageFormat);
            }
            let event = Event::from_json(v[1].to_string())
                .map_err(|_| MessageHandleError::JsonDeserializationFailed)?;
            return Ok(Self::new_event(event));
        }

        // Req
        // ["REQ", <subscription_id>, <filters JSON>...]
        if v[0] == "REQ" {
            if v.len() != 3 {
                return Err(MessageHandleError::InvalidMessageFormat);
            }
            let subscription_id: String = serde_json::from_value(v[1].clone())
                .map_err(|_| MessageHandleError::JsonDeserializationFailed)?;
            let filters: Vec<SubscriptionFilter> = serde_json::from_value(v[2].clone())
                .map_err(|_| MessageHandleError::JsonDeserializationFailed)?;
            return Ok(Self::new_req(subscription_id, filters));
        }

        // Close
        // ["CLOSE", <subscription_id>]
        if v[0] == "CLOSE" {
            if v.len() != 2 {
                return Err(MessageHandleError::InvalidMessageFormat);
            }

            let subscription_id: String = serde_json::from_value(v[1].clone())
                .map_err(|_| MessageHandleError::JsonDeserializationFailed)?;

            return Ok(Self::close(subscription_id));
        }

        Err(MessageHandleError::InvalidMessageFormat)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::str::FromStr;

    use bitcoin::secp256k1::XOnlyPublicKey;

    use crate::{Kind, KindBase};

    #[test]
    fn test_client_message_req() {
        let pk = XOnlyPublicKey::from_str(
            "379e863e8357163b5bce5d2688dc4f1dcc2d505222fb8d74db600f30535dfdfe",
        )
        .unwrap();
        let filters = vec![
            SubscriptionFilter::new().kind(Kind::Base(KindBase::EncryptedDirectMessage)),
            SubscriptionFilter::new().pubkey(pk),
        ];

        let client_req = ClientMessage::new_req("test", filters);
        assert_eq!(
            client_req.to_json(),
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
            client_req.to_json(),
            r##"["REQ","test",{"kinds":[22]},{"#p":["379e863e8357163b5bce5d2688dc4f1dcc2d505222fb8d74db600f30535dfdfe"]}]"##
        );
    }
}
