// Copyright (c) 2021 Paul Miller
// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Event

use std::str::FromStr;

use bitcoin::secp256k1::schnorr::Signature;
use bitcoin::secp256k1::{Message, Secp256k1, XOnlyPublicKey};
use serde::{Deserialize, Deserializer, Serialize};

pub mod builder;
pub mod id;
pub mod kind;
pub mod tag;

pub use self::builder::EventBuilder;
pub use self::id::EventId;
pub use self::kind::Kind;
pub use self::tag::{Marker, Tag, TagKind};
use crate::Timestamp;

/// [`Event`] error
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Invalid signature
    #[error("invalid signature")]
    InvalidSignature,
    /// Error serializing or deserializing JSON data
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    /// Secp256k1 error
    #[error(transparent)]
    Secp256k1(#[from] bitcoin::secp256k1::Error),
    /// Hex decoding error
    #[error(transparent)]
    Hex(#[from] bitcoin::hashes::hex::Error),
}

/// [`Event`] struct
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct Event {
    /// Id
    pub id: EventId,
    /// Author
    pub pubkey: XOnlyPublicKey,
    /// Timestamp (seconds)
    pub created_at: Timestamp,
    /// Kind
    pub kind: Kind,
    /// Vector of [`Tag`]
    pub tags: Vec<Tag>,
    /// Content
    pub content: String,
    /// Signature
    #[serde(deserialize_with = "sig_string")]
    pub sig: Signature,
    /// OpenTimestamps Attestations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ots: Option<String>,
}

fn sig_string<'de, D>(deserializer: D) -> Result<Signature, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    let sig = Signature::from_str(&s);
    sig.map_err(serde::de::Error::custom)
}

impl Event {
    /// Verify event
    pub fn verify(&self) -> Result<(), Error> {
        let secp = Secp256k1::new();
        let id = EventId::new(
            &self.pubkey,
            self.created_at,
            &self.kind,
            &self.tags,
            &self.content,
        );
        let message = Message::from_slice(id.as_bytes())?;
        secp.verify_schnorr(&self.sig, &message, &self.pubkey)
            .map_err(|_| Error::InvalidSignature)
    }

    /// New event from json string
    pub fn from_json<S>(json: S) -> Result<Self, Error>
    where
        S: Into<String>,
    {
        let event: Self = serde_json::from_str(&json.into())?;
        event.verify()?;
        Ok(event)
    }

    /// Get event as json string
    pub fn as_json(&self) -> Result<String, Error> {
        Ok(serde_json::to_string(&self)?)
    }
}

impl Event {
    /// This is just for serde sanity checking
    #[allow(dead_code)]
    pub(crate) fn new_dummy(
        id: &str,
        pubkey: &str,
        created_at: Timestamp,
        kind: u8,
        tags: Vec<Tag>,
        content: &str,
        sig: &str,
    ) -> Result<Self, Error> {
        let id = EventId::from_hex(id).unwrap();
        let pubkey = XOnlyPublicKey::from_str(pubkey)?;
        let kind = serde_json::from_str(&kind.to_string())?;
        let sig = Signature::from_str(sig)?;

        let event = Event {
            id,
            pubkey,
            created_at,
            kind,
            tags,
            content: content.to_string(),
            sig,
            ots: None,
        };

        event.verify()?;

        Ok(event)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::Keys;

    #[test]
    fn test_tags_deser_without_recommended_relay() {
        //The TAG array has dynamic length because the third element(Recommended relay url) is optional
        let sample_event = r#"{"id":"2be17aa3031bdcb006f0fce80c146dea9c1c0268b0af2398bb673365c6444d45","pubkey":"f86c44a2de95d9149b51c6a29afeabba264c18e2fa7c49de93424a0c56947785","created_at":1640839235,"kind":4,"tags":[["p","13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"]],"content":"uRuvYr585B80L6rSJiHocw==?iv=oh6LVqdsYYol3JfFnXTbPA==","sig":"a5d9290ef9659083c490b303eb7ee41356d8778ff19f2f91776c8dc4443388a64ffcf336e61af4c25c05ac3ae952d1ced889ed655b67790891222aaa15b99fdd"}"#;
        let ev_ser = Event::from_json(sample_event).unwrap();
        assert_eq!(ev_ser.as_json().unwrap(), sample_event);
    }

    #[test]
    fn test_custom_kind() {
        let keys = Keys::generate();
        let e: Event = EventBuilder::new(Kind::Custom(123), "my content", &[])
            .to_event(&keys)
            .unwrap();

        let serialized = e.as_json().unwrap();
        let deserialized = Event::from_json(serialized).unwrap();

        assert_eq!(e, deserialized);
        assert_eq!(Kind::Custom(123), e.kind);
        assert_eq!(Kind::Custom(123), deserialized.kind);
    }
}
