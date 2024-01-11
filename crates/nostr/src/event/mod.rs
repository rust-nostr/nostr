// Copyright (c) 2021 Paul Miller
// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Event

use alloc::string::String;
use alloc::vec::Vec;
use core::cmp::Ordering;
use core::fmt;
use core::hash::{Hash, Hasher};
use core::ops::Deref;

use bitcoin::secp256k1::schnorr::Signature;
use bitcoin::secp256k1::{self, Message, Secp256k1, Verification, XOnlyPublicKey};
use serde::ser::SerializeStruct;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
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
pub use self::tag::{Marker, Tag, TagKind};
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
#[derive(Debug, Clone)]
pub struct Event {
    /// Event
    inner: EventIntermediate,
    /// JSON deserialization key order
    deser_order: Vec<String>,
}

impl PartialEq for Event {
    fn eq(&self, other: &Self) -> bool {
        self.inner.eq(&other.inner)
    }
}

impl Eq for Event {}

impl PartialOrd for Event {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Event {
    fn cmp(&self, other: &Self) -> Ordering {
        self.inner.cmp(&other.inner)
    }
}

impl Hash for Event {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.inner.hash(state);
    }
}

impl Deref for Event {
    type Target = EventIntermediate;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Event {
    /// Compose event
    pub fn new<I, S>(
        id: EventId,
        public_key: XOnlyPublicKey,
        created_at: Timestamp,
        kind: Kind,
        tags: I,
        content: S,
        sig: Signature,
    ) -> Self
    where
        I: IntoIterator<Item = Tag>,
        S: Into<String>,
    {
        Self {
            inner: EventIntermediate {
                id,
                pubkey: public_key,
                created_at,
                kind,
                tags: tags.into_iter().collect(),
                content: content.into(),
                sig,
            },
            deser_order: Vec::new(),
        }
    }

    /// Deserialize [`Event`] from [`Value`]
    ///
    /// **This method NOT verify the signature!**
    pub fn from_value(value: Value) -> Result<Self, Error> {
        Ok(serde_json::from_value(value)?)
    }

    /// Get event ID
    pub fn id(&self) -> EventId {
        self.inner.id
    }

    /// Get event author (`pubkey` field)
    pub fn author(&self) -> XOnlyPublicKey {
        self.inner.pubkey
    }

    /// Get event author reference (`pubkey` field)
    pub fn author_ref(&self) -> &XOnlyPublicKey {
        &self.inner.pubkey
    }

    /// Get [Timestamp] of when the event was created
    pub fn created_at(&self) -> Timestamp {
        self.inner.created_at
    }

    /// Get event [Kind]
    pub fn kind(&self) -> Kind {
        self.inner.kind
    }

    /// Get reference to event tags
    pub fn tags(&self) -> &[Tag] {
        &self.inner.tags
    }

    /// Iterate event tags
    pub fn iter_tags(&self) -> impl Iterator<Item = &Tag> {
        self.inner.tags.iter()
    }

    /// Iterate and consume event tags
    pub fn into_iter_tags(self) -> impl Iterator<Item = Tag> {
        self.inner.tags.into_iter()
    }

    /// Get reference to event content
    pub fn content(&self) -> &str {
        &self.inner.content
    }

    /// Get event signature
    pub fn signature(&self) -> Signature {
        self.inner.sig
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
            &self.inner.pubkey,
            self.inner.created_at,
            &self.inner.kind,
            &self.inner.tags,
            &self.inner.content,
        );
        if id == self.inner.id {
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
        let message = Message::from_slice(self.inner.id.as_bytes())?;
        secp.verify_schnorr(&self.inner.sig, &message, &self.inner.pubkey)
            .map_err(|_| Error::InvalidSignature)
    }

    /// Get [`Timestamp`] expiration if set
    pub fn expiration(&self) -> Option<&Timestamp> {
        for tag in self.iter_tags() {
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
        self.inner.kind.is_job_request()
    }

    /// Check if [`Kind`] is a NIP90 job result
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/90.md>
    pub fn is_job_result(&self) -> bool {
        self.inner.kind.is_job_result()
    }

    /// Check if event [`Kind`] is `Regular`
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    pub fn is_regular(&self) -> bool {
        self.inner.kind.is_regular()
    }

    /// Check if event [`Kind`] is `Replaceable`
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    pub fn is_replaceable(&self) -> bool {
        self.inner.kind.is_replaceable()
    }

    /// Check if event [`Kind`] is `Ephemeral`
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    pub fn is_ephemeral(&self) -> bool {
        self.inner.kind.is_ephemeral()
    }

    /// Check if event [`Kind`] is `Parameterized replaceable`
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    pub fn is_parameterized_replaceable(&self) -> bool {
        self.inner.kind.is_parameterized_replaceable()
    }

    /// Extract identifier (`d` tag), if exists.
    pub fn identifier(&self) -> Option<&str> {
        for tag in self.iter_tags() {
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
        self.iter_tags().filter_map(|t| match t {
            Tag::PublicKey { public_key, .. } => Some(public_key),
            _ => None,
        })
    }

    /// Extract event IDs from tags (`e` tag)
    ///
    /// **This method extract ONLY `Tag::Event`**
    pub fn event_ids(&self) -> impl Iterator<Item = &EventId> {
        self.iter_tags().filter_map(|t| match t {
            Tag::Event { event_id, .. } => Some(event_id),
            _ => None,
        })
    }

    /// Extract coordinates from tags (`a` tag)
    pub fn coordinates(&self) -> impl Iterator<Item = Coordinate> + '_ {
        self.iter_tags().filter_map(|t| match t {
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

/// Event Intermediate used for de/serialization of [`Event`]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct EventIntermediate {
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

impl Serialize for Event {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if self.deser_order.is_empty() {
            self.inner.serialize(serializer)
        } else {
            let mut s = serializer.serialize_struct("Event", 7)?;
            for key in self.deser_order.iter() {
                match key.as_str() {
                    "id" => s.serialize_field("id", &self.inner.id)?,
                    "pubkey" => s.serialize_field("pubkey", &self.inner.pubkey)?,
                    "created_at" => s.serialize_field("created_at", &self.inner.created_at)?,
                    "kind" => s.serialize_field("kind", &self.inner.kind)?,
                    "tags" => s.serialize_field("tags", &self.inner.tags)?,
                    "content" => s.serialize_field("content", &self.inner.content)?,
                    "sig" => s.serialize_field("sig", &self.inner.sig)?,
                    _ => return Err(serde::ser::Error::custom(format!("Unknown key: {}", key))),
                }
            }
            s.end()
        }
    }
}

impl<'de> Deserialize<'de> for Event {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value: Value = Value::deserialize(deserializer)?;

        let mut deser_order: Vec<String> = Vec::with_capacity(7);
        if let Value::Object(map) = &value {
            deser_order = map.keys().cloned().collect();
        }

        Ok(Self {
            inner: serde_json::from_value(value).map_err(serde::de::Error::custom)?,
            deser_order,
        })
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
        assert_eq!(Kind::Custom(123), e.kind());
        assert_eq!(Kind::Custom(123), deserialized.kind());
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

    #[test]
    fn test_verify_event_id() {
        let event = Event::from_json(r#"{"content":"","created_at":1698412975,"id":"f55c30722f056e330d8a7a6a9ba1522f7522c0f1ced1c93d78ea833c78a3d6ec","kind":3,"pubkey":"f831caf722214748c72db4829986bd0cbb2bb8b3aeade1c959624a52a9629046","sig":"5092a9ffaecdae7d7794706f085ff5852befdf79df424cc3419bb797bf515ae05d4f19404cb8324b8b4380a4bd497763ac7b0f3b1b63ef4d3baa17e5f5901808","tags":[["p","4ddeb9109a8cd29ba279a637f5ec344f2479ee07df1f4043f3fe26d8948cfef9","",""],["p","bb6fd06e156929649a73e6b278af5e648214a69d88943702f1fb627c02179b95","",""],["p","b8b8210f33888fdbf5cedee9edf13c3e9638612698fe6408aff8609059053420","",""],["p","9dcee4fabcd690dc1da9abdba94afebf82e1e7614f4ea92d61d52ef9cd74e083","",""],["p","3eea9e831fefdaa8df35187a204d82edb589a36b170955ac5ca6b88340befaa0","",""],["p","885238ab4568f271b572bf48b9d6f99fa07644731f288259bd395998ee24754e","",""],["p","568a25c71fba591e39bebe309794d5c15d27dbfa7114cacb9f3586ea1314d126","",""]]}"#).unwrap();
        event.verify_id().unwrap();

        let event = Event::from_json(r#"{"content":"Think about this.\n\nThe most powerful centralized institutions in the world have been replaced by a protocol that protects the individual. #bitcoin\n\nDo you doubt that we can replace everything else?\n\nBullish on the future of humanity\nnostr:nevent1qqs9ljegkuk2m2ewfjlhxy054n6ld5dfngwzuep0ddhs64gc49q0nmqpzdmhxue69uhhyetvv9ukzcnvv5hx7un8qgsw3mfhnrr0l6ll5zzsrtpeufckv2lazc8k3ru5c3wkjtv8vlwngksrqsqqqqqpttgr27","created_at":1703184271,"id":"38acf9b08d06859e49237688a9fd6558c448766f47457236c2331f93538992c6","kind":1,"pubkey":"e8ed3798c6ffebffa08501ac39e271662bfd160f688f94c45d692d8767dd345a","sig":"f76d5ecc8e7de688ac12b9d19edaacdcffb8f0c8fa2a44c00767363af3f04dbc069542ddc5d2f63c94cb5e6ce701589d538cf2db3b1f1211a96596fabb6ecafe","tags":[["e","5fcb28b72cadab2e4cbf7311f4acf5f6d1a99a1c2e642f6b6f0d5518a940f9ec","","mention"],["p","e8ed3798c6ffebffa08501ac39e271662bfd160f688f94c45d692d8767dd345a","","mention"],["t","bitcoin"],["t","bitcoin"]]}"#).unwrap();
        event.verify_id().unwrap();
    }

    // Test only with `std` feature due to `serde_json` preserve_order feature.
    #[test]
    #[cfg(feature = "std")]
    fn test_event_de_serialization_order_preservation() {
        let json = r#"{"content":"uRuvYr585B80L6rSJiHocw==?iv=oh6LVqdsYYol3JfFnXTbPA==","created_at":1640839235,"id":"2be17aa3031bdcb006f0fce80c146dea9c1c0268b0af2398bb673365c6444d45","kind":4,"pubkey":"f86c44a2de95d9149b51c6a29afeabba264c18e2fa7c49de93424a0c56947785","sig":"a5d9290ef9659083c490b303eb7ee41356d8778ff19f2f91776c8dc4443388a64ffcf336e61af4c25c05ac3ae952d1ced889ed655b67790891222aaa15b99fdd","tags":[["p","13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"]]}"#;
        let event = Event::from_json(json).unwrap();
        let reserialized_json = event.as_json();
        assert_eq!(json, reserialized_json);

        let json = r#"{"kind":3,"pubkey":"f831caf722214748c72db4829986bd0cbb2bb8b3aeade1c959624a52a9629046","content":"","created_at":1698412975,"id":"f55c30722f056e330d8a7a6a9ba1522f7522c0f1ced1c93d78ea833c78a3d6ec","sig":"5092a9ffaecdae7d7794706f085ff5852befdf79df424cc3419bb797bf515ae05d4f19404cb8324b8b4380a4bd497763ac7b0f3b1b63ef4d3baa17e5f5901808","tags":[["p","4ddeb9109a8cd29ba279a637f5ec344f2479ee07df1f4043f3fe26d8948cfef9","",""],["p","bb6fd06e156929649a73e6b278af5e648214a69d88943702f1fb627c02179b95","",""],["p","b8b8210f33888fdbf5cedee9edf13c3e9638612698fe6408aff8609059053420","",""],["p","9dcee4fabcd690dc1da9abdba94afebf82e1e7614f4ea92d61d52ef9cd74e083","",""],["p","3eea9e831fefdaa8df35187a204d82edb589a36b170955ac5ca6b88340befaa0","",""],["p","885238ab4568f271b572bf48b9d6f99fa07644731f288259bd395998ee24754e","",""],["p","568a25c71fba591e39bebe309794d5c15d27dbfa7114cacb9f3586ea1314d126","",""]]}"#;
        let event = Event::from_json(json).unwrap();
        let reserialized_json = event.as_json();
        assert_eq!(
            event.deser_order,
            vec![
                "kind",
                "pubkey",
                "content",
                "created_at",
                "id",
                "sig",
                "tags"
            ]
        );
        assert_eq!(json, reserialized_json);

        let json = r#"{"content":"[[\"e\",\"fd40fc62d6349408c5b63d364c1f695b435cc596b58cfaa449519fbc5f2a41a4\"],[\"e\",\"a515bc18a06f0a3561075870f488365e71c5e90aa429a82845e9f7f0d66b6119\"],[\"e\",\"0eb6c73ed0af393a6a2fd9d8200534be064af9d244ef4b211e38503853755b57\"],[\"e\",\"1e8115cb2ba0e14eeb79fcb5ce6cb88f2db59e156aae9ad9302e86e8529e5e7c\"],[\"e\",\"6138b278802611f0685a75d5156f7bd3702a2acab4ba3864665901b1ffd58055\"],[\"e\",\"42105a71922acd113d77d876220fc49aabfa38ba9f34d2267e4f1d45d98b8eaf\"],[\"e\",\"dcd64141fa7af67e61fb28d02085e5c50bb0ccb72270b95e983183179903ef54\"],[\"e\",\"802f72b45a14639477a6ad9d89df9926d59e15d20387ab276dbe92dc48ddc21e\"],[\"e\",\"67ccd79069e27330480e1111f939c0770548e4222f4b5bcdf87ea9ec09e37abf\"],[\"e\",\"c45f94f3c8648536333b657287f0820c4ff1857fb1849a8ce8a541762f233063\"],[\"e\",\"afd22572b31ab14d0c6f65880e626d8e7fe20407ef1486e3ef78820be37e27d8\"],[\"e\",\"bd6a1a577ecfc5ba2ac5a391cae8f21a6238a7ad61a4ebcdd2a44ca488dd03c9\"],[\"e\",\"044ac6073a9cf1b723028a7828fdca098bcd0b79e5e58c21e2372c6b48bd67ca\"],[\"e\",\"2585dcecf6033f82d689a6456af2c82e7d5d9d9e64f90e2c7e86a80eb7dc765b\"],[\"e\",\"08a579677eee0b1796060dbd1e71dcc7ad0937be64ca278b61ef4c3dde149252\"],[\"e\",\"3ed3eaa26cdd1a35808775a8f0c6bd432c0dd1b9c2bc326c9dd249ecf2fe0270\"],[\"e\",\"a2bc2e1149d952a9af202529f3bdd4e8f11a9fda1bd2ad5c6dbbc8b83a1ebc2f\"],[\"e\",\"82e5c6ee536832ababb8eba47e1255d8b1820ca360d2c467f2f32fc610fe3047\"],[\"e\",\"1990b084eb9d0d524ff52f7fb2f0e7f1a1fee977b893c191af7893f53acf7d05\"],[\"e\",\"8df981ac84ca018c7972874770dbf19996f28e9c785eac473bab246e2ad92661\"],[\"e\",\"b975c677ee7517d9124ec8d69d3fafee7ddf6b1d291cc19dffd2678c2241f095\"],[\"e\",\"972599d1139da7e33dc39f049656935ae3b576492f1c535a0eda8d10b1eeb27d\"],[\"e\",\"eaaa6e0cda6315fa30841e9124a526c23dc631fcbf0ffc5e166bbd41d3585efa\"],[\"e\",\"e5eb71fe3dc364d51b6bd6cef73009704df5ee90674a54cb16168e78bbf8fa95\"],[\"e\",\"a49dd0610479b1d81b26f84b949d88d19abc4c3a6b86a1b6501ff393e9618700\"]]","created_at":1701278715,"id":"d05e7ae9271fe2d8968cccb67c01e3458dbafa4a415e306d49b22729b088c8a1","kind":6300,"pubkey":"6b37d5dc88c1cbd32d75b713f6d4c2f7766276f51c9337af9d32c8d715cc1b93","sig":"ee590cf98548039ccbeccb246e55310ad14bb0a307452dacca3f9d1760ac5fdb22d1f1bd932c5fc41d97b8cc16d82719c8ad24440b8d99c38ff2eb0486576253","tags":[["status","success"],["request","{\"created_at\":1701278699,\"content\":\"\",\"tags\":[[\"relays\",\"wss://pablof7z.nostr1.com\",\"wss://purplepag.es\",\"wss://nos.lol\",\"wss://relay.f7z.io\",\"wss://relay.damus.io\",\"wss://relay.snort.social\",\"wss://offchain.pub/\",\"wss://nostr-pub.wellorder.net\"],[\"output\",\"text/plain\"],[\"param\",\"user\",\"99bb5591c9116600f845107d31f9b59e2f7c7e09a1ff802e84f1d43da557ca64\"],[\"relays\",\"wss://relay.damus.io\",\"wss://offchain.pub/\",\"wss://pablof7z.nostr1.com\",\"wss://nos.lol\"]],\"kind\":5300,\"pubkey\":\"99bb5591c9116600f845107d31f9b59e2f7c7e09a1ff802e84f1d43da557ca64\",\"id\":\"5635e5dd930b3c831f6ab1e348bb488f3c9aca2f13190e93ab5e5e1e1ba1835e\",\"sig\":\"babbf39cf1875271d99be7319667f6f83349ffa0ad9262a7ca4719b60601e19642763733840fd7cbef2e883a19fd7829102709fb6af25a6d978b82fba2673140\"}"],["e","5635e5dd930b3c831f6ab1e348bb488f3c9aca2f13190e93ab5e5e1e1ba1835e"],["p","99bb5591c9116600f845107d31f9b59e2f7c7e09a1ff802e84f1d43da557ca64"],["p","99bb5591c9116600f845107d31f9b59e2f7c7e09a1ff802e84f1d43da557ca64"]]}"#;
        let event = Event::from_json(json).unwrap();
        let reserialized_json = event.as_json();
        assert_eq!(json, reserialized_json);
    }
}

#[cfg(bench)]
mod benches {
    use test::{black_box, Bencher};

    use super::*;

    #[bench]
    pub fn deserialize_event(bh: &mut Bencher) {
        let json = r#"{"content":"uRuvYr585B80L6rSJiHocw==?iv=oh6LVqdsYYol3JfFnXTbPA==","created_at":1640839235,"id":"2be17aa3031bdcb006f0fce80c146dea9c1c0268b0af2398bb673365c6444d45","kind":4,"pubkey":"f86c44a2de95d9149b51c6a29afeabba264c18e2fa7c49de93424a0c56947785","sig":"a5d9290ef9659083c490b303eb7ee41356d8778ff19f2f91776c8dc4443388a64ffcf336e61af4c25c05ac3ae952d1ced889ed655b67790891222aaa15b99fdd","tags":[["p","13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"]]}"#;
        bh.iter(|| {
            black_box(Event::from_json(json)).unwrap();
        });
    }

    #[bench]
    pub fn serialize_event(bh: &mut Bencher) {
        let json = r#"{"content":"uRuvYr585B80L6rSJiHocw==?iv=oh6LVqdsYYol3JfFnXTbPA==","created_at":1640839235,"id":"2be17aa3031bdcb006f0fce80c146dea9c1c0268b0af2398bb673365c6444d45","kind":4,"pubkey":"f86c44a2de95d9149b51c6a29afeabba264c18e2fa7c49de93424a0c56947785","sig":"a5d9290ef9659083c490b303eb7ee41356d8778ff19f2f91776c8dc4443388a64ffcf336e61af4c25c05ac3ae952d1ced889ed655b67790891222aaa15b99fdd","tags":[["p","13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"]]}"#;
        let event = Event::from_json(json).unwrap();
        bh.iter(|| {
            black_box(event.as_json());
        });
    }
}
