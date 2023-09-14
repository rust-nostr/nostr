// Copyright (c) 2021 Paul Miller
// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Event

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt;
use core::str::FromStr;

use bitcoin::secp256k1::schnorr::Signature;
use bitcoin::secp256k1::{self, Message, Secp256k1, Verification, XOnlyPublicKey};
use serde_json::Value;

pub mod builder;
pub mod id;
pub mod kind;
pub mod tag;
pub mod unsigned;

pub use self::builder::EventBuilder;
pub use self::id::EventId;
pub use self::kind::Kind;
pub use self::tag::{Marker, Tag, TagKind};
pub use self::unsigned::UnsignedEvent;
#[cfg(feature = "std")]
use crate::types::time::Instant;
use crate::types::time::TimeSupplier;
use crate::Timestamp;
#[cfg(feature = "std")]
use crate::SECP256K1;

/// [`Event`] error
#[derive(Debug)]
pub enum Error {
    /// Invalid signature
    InvalidSignature,
    /// Invalid event id
    InvalidId,
    /// Error serializing or deserializing JSON data
    Json(serde_json::Error),
    /// Secp256k1 error
    Secp256k1(secp256k1::Error),
    /// Hex decoding error
    Hex(bitcoin::hashes::hex::Error),
    /// OpenTimestamps error
    #[cfg(feature = "nip03")]
    OpenTimestamps(nostr_ots::Error),
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSignature => write!(f, "Invalid signature"),
            Self::InvalidId => write!(f, "Invalid event id"),
            Self::Json(e) => write!(f, "Json: {e}"),
            Self::Secp256k1(e) => write!(f, "Secp256k1: {e}"),
            Self::Hex(e) => write!(f, "Hex: {e}"),
            #[cfg(feature = "nip03")]
            Self::OpenTimestamps(e) => write!(f, "NIP03: {e}"),
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}

impl From<secp256k1::Error> for Error {
    fn from(e: secp256k1::Error) -> Self {
        Self::Secp256k1(e)
    }
}

impl From<bitcoin::hashes::hex::Error> for Error {
    fn from(e: bitcoin::hashes::hex::Error) -> Self {
        Self::Hex(e)
    }
}

#[cfg(feature = "nip03")]
impl From<nostr_ots::Error> for Error {
    fn from(e: nostr_ots::Error) -> Self {
        Self::OpenTimestamps(e)
    }
}

/// [`Event`] struct
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
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
    pub sig: Signature,
    /// OpenTimestamps Attestations
    #[cfg(feature = "nip03")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ots: Option<String>,
}

impl Event {
    /// Verify event
    #[cfg(feature = "std")]
    pub fn verify(&self) -> Result<(), Error> {
        self.verify_with_ctx(&SECP256K1)
    }

    /// Verify event
    pub fn verify_with_ctx<C>(&self, secp: &Secp256k1<C>) -> Result<(), Error>
    where
        C: Verification,
    {
        let id = EventId::new(
            &self.pubkey,
            self.created_at,
            &self.kind,
            &self.tags,
            &self.content,
        );
        if id == self.id {
            let message = Message::from_slice(id.as_bytes())?;
            secp.verify_schnorr(&self.sig, &message, &self.pubkey)
                .map_err(|_| Error::InvalidSignature)
        } else {
            Err(Error::InvalidId)
        }
    }

    /// New event from [`Value`]
    #[cfg(feature = "std")]
    pub fn from_value(value: Value) -> Result<Self, Error> {
        Self::from_value_with_ctx(&SECP256K1, value)
    }

    /// New event from [`Value`]
    pub fn from_value_with_ctx<C>(secp: &Secp256k1<C>, value: Value) -> Result<Self, Error>
    where
        C: Verification,
    {
        let event: Self = serde_json::from_value(value)?;
        event.verify_with_ctx(secp)?;
        Ok(event)
    }

    /// New event from json string
    #[cfg(feature = "std")]
    pub fn from_json<S>(json: S) -> Result<Self, Error>
    where
        S: Into<String>,
    {
        Self::from_json_with_ctx(&SECP256K1, json)
    }

    /// New event from json string
    pub fn from_json_with_ctx<C, S>(secp: &Secp256k1<C>, json: S) -> Result<Self, Error>
    where
        C: Verification,
        S: Into<String>,
    {
        let event: Self = serde_json::from_str(&json.into())?;
        event.verify_with_ctx(secp)?;
        Ok(event)
    }

    /// Get event as json string
    pub fn as_json(&self) -> String {
        serde_json::json!(self).to_string()
    }

    /// Returns `true` if the event has an expiration tag that is expired.
    /// If an event has no `Expiration` tag, then it will return `false`.
    #[cfg(feature = "std")]
    pub fn is_expired(&self) -> bool {
        let now: Instant = Instant::now();
        self.is_expired_with_supplier(&now)
    }

    /// Returns `true` if the event has an expiration tag that is expired.
    /// If an event has no `Expiration` tag, then it will return `false`.
    pub fn is_expired_with_supplier<T>(&self, supplier: &T) -> bool
    where
        T: TimeSupplier,
    {
        let now: Timestamp = Timestamp::now_with_supplier(supplier);
        for tag in self.tags.iter() {
            if let Tag::Expiration(timestamp) = tag {
                return timestamp < &now;
            }
        }
        false
    }

    /// Timestamp this event with OpenTimestamps, according to NIP-03
    #[cfg(feature = "nip03")]
    pub fn timestamp(&mut self) -> Result<(), Error> {
        let ots = nostr_ots::timestamp_event(&self.id.to_hex())?;
        self.ots = Some(ots);
        Ok(())
    }

    /// Check if event [`Kind`] is `Regular`
    pub fn is_regular(&self) -> bool {
        self.kind.is_regular()
    }

    /// Check if event [`Kind`] is `Replaceable`
    pub fn is_replaceable(&self) -> bool {
        self.kind.is_replaceable()
    }

    /// Check if event [`Kind`] is `Ephemeral`
    pub fn is_ephemeral(&self) -> bool {
        self.kind.is_ephemeral()
    }

    /// Check if event [`Kind`] is `Parameterized replaceable`
    pub fn is_parameterized_replaceable(&self) -> bool {
        self.kind.is_parameterized_replaceable()
    }
}

impl Event {
    /// This is just for serde sanity checking
    #[allow(dead_code)]
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new_dummy<C>(
        secp: &Secp256k1<C>,
        id: &str,
        pubkey: &str,
        created_at: Timestamp,
        kind: u64,
        tags: Vec<Tag>,
        content: &str,
        sig: &str,
    ) -> Result<Self, Error>
    where
        C: Verification,
    {
        let id = EventId::from_hex(id).unwrap();
        let pubkey = XOnlyPublicKey::from_str(pubkey)?;
        let kind = Kind::from(kind);
        let sig = Signature::from_str(sig)?;

        let event = Event {
            id,
            pubkey,
            created_at,
            kind,
            tags,
            content: content.to_string(),
            sig,
            #[cfg(feature = "nip03")]
            ots: None,
        };

        event.verify_with_ctx(secp)?;

        Ok(event)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "std")]
    use crate::Keys;

    #[test]
    fn test_tags_deser_without_recommended_relay() {
        let secp = Secp256k1::new();

        //The TAG array has dynamic length because the third element(Recommended relay url) is optional
        let sample_event = r#"{"content":"uRuvYr585B80L6rSJiHocw==?iv=oh6LVqdsYYol3JfFnXTbPA==","created_at":1640839235,"id":"2be17aa3031bdcb006f0fce80c146dea9c1c0268b0af2398bb673365c6444d45","kind":4,"pubkey":"f86c44a2de95d9149b51c6a29afeabba264c18e2fa7c49de93424a0c56947785","sig":"a5d9290ef9659083c490b303eb7ee41356d8778ff19f2f91776c8dc4443388a64ffcf336e61af4c25c05ac3ae952d1ced889ed655b67790891222aaa15b99fdd","tags":[["p","13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"]]}"#;
        let ev_ser = Event::from_json_with_ctx(&secp, sample_event).unwrap();
        assert_eq!(ev_ser.as_json(), sample_event);
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_custom_kind() {
        let keys = Keys::generate();
        let e: Event = EventBuilder::new(Kind::Custom(123), "my content", &[])
            .to_event(&keys)
            .unwrap();

        let serialized = e.as_json();
        let deserialized = Event::from_json(serialized).unwrap();

        assert_eq!(e, deserialized);
        assert_eq!(Kind::Custom(123), e.kind);
        assert_eq!(Kind::Custom(123), deserialized.kind);
    }
    #[test]
    #[cfg(feature = "std")]
    fn test_event_expired() {
        let my_keys = Keys::generate();
        let event = EventBuilder::new_text_note(
            "my content",
            &[Tag::Expiration(Timestamp::from(1600000000))],
        )
        .to_event(&my_keys)
        .unwrap();

        assert!(&event.is_expired());
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_event_not_expired() {
        let now = Timestamp::now().as_i64();

        // To make sure it is never considered expired
        let expiry_date: u64 = (now * 2).try_into().unwrap();

        let my_keys = Keys::generate();
        let event = EventBuilder::new_text_note(
            "my content",
            &[Tag::Expiration(Timestamp::from(expiry_date))],
        )
        .to_event(&my_keys)
        .unwrap();

        assert!(!&event.is_expired());
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_event_without_expiration_tag() {
        let my_keys = Keys::generate();
        let event = EventBuilder::new_text_note("my content", &[])
            .to_event(&my_keys)
            .unwrap();
        assert!(!&event.is_expired());
    }
}
