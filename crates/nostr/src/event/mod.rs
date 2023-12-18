// Copyright (c) 2021 Paul Miller
// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Event

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use bitcoin::secp256k1::schnorr::Signature;
use bitcoin::secp256k1::{self, Message, Secp256k1, Verification, XOnlyPublicKey};
use serde_json::Value;

pub mod builder;
pub mod id;
pub mod kind;
pub mod partial;
pub mod tag;
pub mod unsigned;

pub use self::builder::EventBuilder;
pub use self::id::EventId;
pub use self::kind::Kind;
pub use self::partial::{MissingPartialEvent, PartialEvent};
pub use self::tag::{Marker, Tag, TagIndexValues, TagIndexes, TagKind};
pub use self::unsigned::UnsignedEvent;
use crate::nips::nip01::Coordinate;
#[cfg(feature = "std")]
use crate::types::time::Instant;
use crate::types::time::TimeSupplier;
#[cfg(feature = "std")]
use crate::SECP256K1;
use crate::{JsonUtil, Timestamp};

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
}

impl Event {
    /// Deserialize [`Event`] from [`Value`]
    ///
    /// **This method NOT verify the signature!**
    pub fn from_value(value: Value) -> Result<Self, Error> {
        Ok(serde_json::from_value(value)?)
    }

    /// Verify both [`EventId`] and [`Signature`]
    #[cfg(feature = "std")]
    pub fn verify(&self) -> Result<(), Error> {
        self.verify_with_ctx(&SECP256K1)
    }

    /// Verify [`EventId`] and [`Signature`]
    pub fn verify_with_ctx<C>(&self, secp: &Secp256k1<C>) -> Result<(), Error>
    where
        C: Verification,
    {
        // Verify ID
        self.verify_id()?;

        // Verify signature
        self.verify_signature_with_ctx(secp)
    }

    /// Verify if the [`EventId`] it's composed correctly
    pub fn verify_id(&self) -> Result<(), Error> {
        let id: EventId = EventId::new(
            &self.pubkey,
            self.created_at,
            &self.kind,
            &self.tags,
            &self.content,
        );
        if id == self.id {
            Ok(())
        } else {
            Err(Error::InvalidId)
        }
    }

    /// Verify only event [`Signature`]
    #[cfg(feature = "std")]
    pub fn verify_signature(&self) -> Result<(), Error> {
        self.verify_with_ctx(&SECP256K1)
    }

    /// Verify event [`Signature`]
    pub fn verify_signature_with_ctx<C>(&self, secp: &Secp256k1<C>) -> Result<(), Error>
    where
        C: Verification,
    {
        let message = Message::from_slice(self.id.as_bytes())?;
        secp.verify_schnorr(&self.sig, &message, &self.pubkey)
            .map_err(|_| Error::InvalidSignature)
    }

    /// Get [`Timestamp`] expiration if set
    pub fn expiration(&self) -> Option<&Timestamp> {
        for tag in self.tags.iter() {
            if let Tag::Expiration(timestamp) = tag {
                return Some(timestamp);
            }
        }
        None
    }

    /// Returns `true` if the event has an expiration tag that is expired.
    /// If an event has no `Expiration` tag, then it will return `false`.
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/40.md>
    #[cfg(feature = "std")]
    pub fn is_expired(&self) -> bool {
        let now: Instant = Instant::now();
        self.is_expired_with_supplier(&now)
    }

    /// Returns `true` if the event has an expiration tag that is expired.
    /// If an event has no `Expiration` tag, then it will return `false`.
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/40.md>
    pub fn is_expired_with_supplier<T>(&self, supplier: &T) -> bool
    where
        T: TimeSupplier,
    {
        if let Some(timestamp) = self.expiration() {
            let now: Timestamp = Timestamp::now_with_supplier(supplier);
            return timestamp < &now;
        }
        false
    }

    /// Check if [`Kind`] is a NIP90 job request
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/90.md>
    pub fn is_job_request(&self) -> bool {
        self.kind.is_job_request()
    }

    /// Check if [`Kind`] is a NIP90 job result
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/90.md>
    pub fn is_job_result(&self) -> bool {
        self.kind.is_job_result()
    }

    /// Check if event [`Kind`] is `Regular`
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    pub fn is_regular(&self) -> bool {
        self.kind.is_regular()
    }

    /// Check if event [`Kind`] is `Replaceable`
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    pub fn is_replaceable(&self) -> bool {
        self.kind.is_replaceable()
    }

    /// Check if event [`Kind`] is `Ephemeral`
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    pub fn is_ephemeral(&self) -> bool {
        self.kind.is_ephemeral()
    }

    /// Check if event [`Kind`] is `Parameterized replaceable`
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    pub fn is_parameterized_replaceable(&self) -> bool {
        self.kind.is_parameterized_replaceable()
    }

    /// Extract identifier (`d` tag), if exists.
    pub fn identifier(&self) -> Option<&str> {
        for tag in self.tags.iter() {
            if let Tag::Identifier(id) = tag {
                return Some(id);
            }
        }
        None
    }

    /// Extract public keys from tags (`p` tag)
    ///
    /// **This method extract ONLY `Tag::PublicKey`**
    pub fn public_keys(&self) -> impl Iterator<Item = &XOnlyPublicKey> {
        self.tags.iter().filter_map(|t| match t {
            Tag::PublicKey { public_key, .. } => Some(public_key),
            _ => None,
        })
    }

    /// Extract event IDs from tags (`e` tag)
    ///
    /// **This method extract ONLY `Tag::Event`**
    pub fn event_ids(&self) -> impl Iterator<Item = &EventId> {
        self.tags.iter().filter_map(|t| match t {
            Tag::Event { event_id, .. } => Some(event_id),
            _ => None,
        })
    }

    /// Extract coordinates from tags (`a` tag)
    pub fn coordinates(&self) -> impl Iterator<Item = Coordinate> + '_ {
        self.tags.iter().filter_map(|t| match t {
            Tag::A {
                kind,
                public_key,
                identifier,
                ..
            } => Some(Coordinate {
                kind: *kind,
                pubkey: *public_key,
                identifier: identifier.clone(),
                relays: Vec::new(),
            }),
            _ => None,
        })
    }

    /// Build tags index
    pub fn build_tags_index(&self) -> TagIndexes {
        TagIndexes::from(self.tags.iter().map(|t| t.as_vec()))
    }
}

impl JsonUtil for Event {
    type Err = Error;

    /// Deserialize [`Event`] from JSON
    ///
    /// **This method NOT verify the signature!**
    fn from_json<T>(json: T) -> Result<Self, Self::Err>
    where
        T: AsRef<[u8]>,
    {
        Ok(serde_json::from_slice(json.as_ref())?)
    }
}

#[cfg(test)]
impl Event {
    /// This is just for serde sanity checking
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new_dummy<S>(
        id: &str,
        pubkey: &str,
        created_at: Timestamp,
        kind: u64,
        tags: Vec<Tag>,
        content: S,
        sig: &str,
    ) -> Self
    where
        S: Into<String>,
    {
        use core::str::FromStr;

        Self {
            id: EventId::from_hex(id).unwrap(),
            pubkey: XOnlyPublicKey::from_str(pubkey).unwrap(),
            created_at,
            kind: Kind::from(kind),
            tags,
            content: content.into(),
            sig: Signature::from_str(sig).unwrap(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "std")]
    use crate::Keys;

    #[test]
    fn test_tags_deser_without_recommended_relay() {
        //The TAG array has dynamic length because the third element(Recommended relay url) is optional
        let sample_event = r#"{"content":"uRuvYr585B80L6rSJiHocw==?iv=oh6LVqdsYYol3JfFnXTbPA==","created_at":1640839235,"id":"2be17aa3031bdcb006f0fce80c146dea9c1c0268b0af2398bb673365c6444d45","kind":4,"pubkey":"f86c44a2de95d9149b51c6a29afeabba264c18e2fa7c49de93424a0c56947785","sig":"a5d9290ef9659083c490b303eb7ee41356d8778ff19f2f91776c8dc4443388a64ffcf336e61af4c25c05ac3ae952d1ced889ed655b67790891222aaa15b99fdd","tags":[["p","13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"]]}"#;
        let ev_ser = Event::from_json(sample_event).unwrap();
        assert_eq!(ev_ser.as_json(), sample_event);
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_custom_kind() {
        let keys = Keys::generate();
        let e: Event = EventBuilder::new(Kind::Custom(123), "my content", [])
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
            [Tag::Expiration(Timestamp::from(1600000000))],
        )
        .to_event(&my_keys)
        .unwrap();

        assert!(&event.is_expired());
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_event_not_expired() {
        let now = Timestamp::now();
        let expiry_date: u64 = now.as_u64() * 2;

        let my_keys = Keys::generate();
        let event = EventBuilder::new_text_note(
            "my content",
            [Tag::Expiration(Timestamp::from(expiry_date))],
        )
        .to_event(&my_keys)
        .unwrap();

        assert!(!&event.is_expired());
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_event_without_expiration_tag() {
        let my_keys = Keys::generate();
        let event = EventBuilder::new_text_note("my content", [])
            .to_event(&my_keys)
            .unwrap();
        assert!(!&event.is_expired());
    }
}
