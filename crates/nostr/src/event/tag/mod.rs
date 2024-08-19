// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Tag

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::cmp::Ordering;
use core::fmt;
use core::hash::{Hash, Hasher};

#[cfg(feature = "std")]
use once_cell::sync::OnceCell; // TODO: when MSRV will be >= 1.70.0, use `std::cell::OnceLock` instead and remove `once_cell` dep.
#[cfg(not(feature = "std"))]
use once_cell::unsync::OnceCell; // TODO: when MSRV will be >= 1.70.0, use `core::cell::OnceCell` instead and remove `once_cell` dep.
use serde::de::Error as DeserializerError;
use serde::ser::SerializeSeq;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

mod error;
pub mod kind;
pub mod standard;

pub use self::error::Error;
pub use self::kind::TagKind;
pub use self::standard::TagStandard;
use super::id::EventId;
use crate::nips::nip01::Coordinate;
use crate::nips::nip10::Marker;
use crate::nips::nip56::Report;
use crate::nips::nip65::RelayMetadata;
use crate::types::url::Url;
use crate::{ImageDimensions, PublicKey, SingleLetterTag, Timestamp, UncheckedUrl};

/// Tag
#[derive(Clone)]
pub struct Tag {
    buf: Vec<String>,
    standardized: OnceCell<Option<TagStandard>>,
}

impl fmt::Debug for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Tag").field(&self.buf).finish()
    }
}

impl PartialEq for Tag {
    fn eq(&self, other: &Self) -> bool {
        self.buf == other.buf
    }
}

impl Eq for Tag {}

impl PartialOrd for Tag {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Tag {
    fn cmp(&self, other: &Self) -> Ordering {
        self.buf.cmp(&other.buf)
    }
}

impl Hash for Tag {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.buf.hash(state);
    }
}

impl Tag {
    #[inline]
    fn new(buf: Vec<String>, standardized: Option<TagStandard>) -> Self {
        Self {
            buf,
            standardized: OnceCell::from(standardized),
        }
    }

    #[inline]
    fn new_with_empty_cell(buf: Vec<String>) -> Self {
        Self {
            buf,
            standardized: OnceCell::new(),
        }
    }

    /// Parse tag
    ///
    /// Return error if the tag is empty!
    pub fn parse<S>(tag: &[S]) -> Result<Self, Error>
    where
        S: AsRef<str>,
    {
        // Check if it's empty
        if tag.is_empty() {
            return Err(Error::EmptyTag);
        }

        // NOT USE `Self::new`!
        Ok(Self::new_with_empty_cell(
            tag.iter().map(|v| v.as_ref().to_string()).collect(),
        ))
    }

    /// Construct from standardized tag
    #[inline]
    pub fn from_standardized(standardized: TagStandard) -> Self {
        Self::new(standardized.clone().to_vec(), Some(standardized))
    }

    /// Construct from standardized tag without initialize cell (avoid a clone)
    #[inline]
    pub fn from_standardized_without_cell(standardized: TagStandard) -> Self {
        Self::new_with_empty_cell(standardized.to_vec())
    }

    /// Get tag kind
    #[inline]
    pub fn kind(&self) -> TagKind {
        // SAFETY: `buf` must not be empty, checked during parsing.
        let key: &str = &self.buf[0];
        TagKind::from(key)
    }

    /// Return the **first** tag value (index `1`), if exists.
    #[inline]
    pub fn content(&self) -> Option<&str> {
        self.buf.get(1).map(|s| s.as_str())
    }

    /// Get [SingleLetterTag]
    #[inline]
    pub fn single_letter_tag(&self) -> Option<SingleLetterTag> {
        match self.kind() {
            TagKind::SingleLetter(s) => Some(s),
            _ => None,
        }
    }

    /// Get reference of standardized tag
    #[inline]
    pub fn as_standardized(&self) -> Option<&TagStandard> {
        self.standardized
            .get_or_init(|| TagStandard::parse(self.as_vec()).ok())
            .as_ref()
    }

    /// Consume tag and get standardized tag
    pub fn to_standardized(self) -> Option<TagStandard> {
        match self.standardized.into_inner() {
            Some(inner) => inner,
            None => TagStandard::parse(&self.buf).ok(),
        }
    }

    /// Get reference of array of strings
    #[inline]
    pub fn as_vec(&self) -> &[String] {
        &self.buf
    }

    /// Consume tag and return array of strings
    #[inline]
    pub fn to_vec(self) -> Vec<String> {
        self.buf
    }

    /// Compose `["e", "<event-id">]`
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    #[inline]
    pub fn event(event_id: EventId) -> Self {
        Self::from_standardized_without_cell(TagStandard::event(event_id))
    }

    /// Compose `["p", "<public-key>"]` tag
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    #[inline]
    pub fn public_key(public_key: PublicKey) -> Self {
        Self::from_standardized_without_cell(TagStandard::public_key(public_key))
    }

    /// Compose `["d", "<identifier>"]` tag
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    #[inline]
    pub fn identifier<T>(identifier: T) -> Self
    where
        T: Into<String>,
    {
        Self::from_standardized_without_cell(TagStandard::Identifier(identifier.into()))
    }

    /// Compose `["a", "<coordinate>"]` tag
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    #[inline]
    pub fn coordinate(coordinate: Coordinate) -> Self {
        Self::from_standardized_without_cell(TagStandard::Coordinate {
            coordinate,
            relay_url: None,
        })
    }

    /// Compose `["nonce", "<nonce>", "<difficulty>"]` tag
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/13.md>
    #[inline]
    pub fn pow(nonce: u128, difficulty: u8) -> Self {
        Self::from_standardized_without_cell(TagStandard::POW { nonce, difficulty })
    }

    /// Compose `["expiration", "<timestamp>"]` tag
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/40.md>
    #[inline]
    pub fn expiration(timestamp: Timestamp) -> Self {
        Self::from_standardized_without_cell(TagStandard::Expiration(timestamp))
    }

    /// Compose `["e", "<event-id>", "<report>"]` tag
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/56.md>
    #[inline]
    pub fn event_report(event_id: EventId, report: Report) -> Self {
        Self::from_standardized_without_cell(TagStandard::EventReport(event_id, report))
    }

    /// Compose `["p", "<public-key>", "<report>"]` tag
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/56.md>
    #[inline]
    pub fn public_key_report(public_key: PublicKey, report: Report) -> Self {
        Self::from_standardized_without_cell(TagStandard::PublicKeyReport(public_key, report))
    }

    /// Compose `["r", "<relay-url>", "<metadata>"]` tag
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/65.md>
    #[inline]
    pub fn relay_metadata(relay_url: Url, metadata: Option<RelayMetadata>) -> Self {
        Self::from_standardized_without_cell(TagStandard::RelayMetadata {
            relay_url,
            metadata,
        })
    }

    /// Compose `["t", "<hashtag>"]` tag
    #[inline]
    pub fn hashtag<T>(hashtag: T) -> Self
    where
        T: Into<String>,
    {
        Self::from_standardized_without_cell(TagStandard::Hashtag(hashtag.into()))
    }

    /// Compose `["title", "<title>"]` tag
    #[inline]
    pub fn title<T>(title: T) -> Self
    where
        T: Into<String>,
    {
        Self::from_standardized_without_cell(TagStandard::Title(title.into()))
    }

    /// Compose image tag
    #[inline]
    pub fn image(url: UncheckedUrl, dimensions: Option<ImageDimensions>) -> Self {
        Self::from_standardized_without_cell(TagStandard::Image(url, dimensions))
    }

    /// Compose `["description", "<description>"]` tag
    #[inline]
    pub fn description<T>(description: T) -> Self
    where
        T: Into<String>,
    {
        Self::from_standardized_without_cell(TagStandard::Description(description.into()))
    }

    /// Protected event
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/70.md>
    #[inline]
    pub fn protected() -> Self {
        Self::from_standardized_without_cell(TagStandard::Protected)
    }

    /// A short human-readable plaintext summary of what that event is about
    ///
    /// JSON: `["alt", "<summary>"]`
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/31.md>
    #[inline]
    pub fn alt<T>(summary: T) -> Self
    where
        T: Into<String>,
    {
        Self::from_standardized_without_cell(TagStandard::Alt(summary.into()))
    }

    /// Compose custom tag
    ///
    /// JSON: `["<kind>", "<value-1>", "<value-2>", ...]`
    pub fn custom<I, S>(kind: TagKind, values: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        // Compose tag
        let mut buf: Vec<String> = Vec::with_capacity(1);
        buf.push(kind.to_string());
        buf.extend(values.into_iter().map(|v| v.into()));

        // NOT USE `Self::new`!
        Self::new_with_empty_cell(buf)
    }

    /// Check if is a standard event tag with `root` marker
    #[inline]
    pub fn is_root(&self) -> bool {
        matches!(
            self.as_standardized(),
            Some(TagStandard::Event {
                marker: Some(Marker::Root),
                ..
            })
        )
    }

    /// Check if is a standard event tag with `reply` marker
    #[inline]
    pub fn is_reply(&self) -> bool {
        matches!(
            self.as_standardized(),
            Some(TagStandard::Event {
                marker: Some(Marker::Reply),
                ..
            })
        )
    }

    /// Check if it's a protected event tag
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/70.md>
    #[inline]
    pub fn is_protected(&self) -> bool {
        matches!(self.as_standardized(), Some(TagStandard::Protected))
    }
}

impl Serialize for Tag {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.buf.len()))?;
        for element in self.buf.iter() {
            seq.serialize_element(&element)?;
        }
        seq.end()
    }
}

impl<'de> Deserialize<'de> for Tag {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        type Data = Vec<String>;
        let tag: Vec<String> = Data::deserialize(deserializer)?;
        Self::parse(&tag).map_err(DeserializerError::custom)
    }
}

impl From<TagStandard> for Tag {
    #[inline(always)]
    fn from(standard: TagStandard) -> Self {
        Self::from_standardized_without_cell(standard)
    }
}

#[cfg(test)]
mod tests {
    use alloc::borrow::Cow;
    use core::str::FromStr;

    use bitcoin::secp256k1::schnorr::Signature;

    use super::*;
    use crate::nips::nip26::Conditions;
    use crate::nips::nip39::{ExternalIdentity, Identity};
    use crate::nips::nip53::LiveEventMarker;
    use crate::{Alphabet, Event, JsonUtil, Kind, Timestamp, UncheckedUrl};

    #[test]
    fn test_tag_match_standardized() {
        let tag: Tag = Tag::parse(&["d", "bravery"]).unwrap();
        assert_eq!(
            tag.as_standardized(),
            Some(&TagStandard::Identifier(String::from("bravery")))
        );

        let tag: Tag = Tag::parse(&["d", "test"]).unwrap();
        assert_eq!(
            tag.to_standardized(),
            Some(TagStandard::Identifier(String::from("test")))
        );
    }

    #[test]
    fn test_tag_standard_is_reply() {
        let tag = TagStandard::Relay(UncheckedUrl::new("wss://relay.damus.io"));
        assert!(!tag.is_reply());

        let tag = TagStandard::Event {
            event_id: EventId::from_hex(
                "2be17aa3031bdcb006f0fce80c146dea9c1c0268b0af2398bb673365c6444d45",
            )
            .unwrap(),
            relay_url: None,
            marker: Some(Marker::Reply),
            public_key: None,
        };
        assert!(tag.is_reply());

        let tag = TagStandard::Event {
            event_id: EventId::from_hex(
                "2be17aa3031bdcb006f0fce80c146dea9c1c0268b0af2398bb673365c6444d45",
            )
            .unwrap(),
            relay_url: None,
            marker: Some(Marker::Root),
            public_key: None,
        };
        assert!(!tag.is_reply());
    }

    #[test]
    fn test_extract_tag_content() {
        let t: Tag = Tag::parse(&["aaaaaa", "bbbbbb"]).unwrap();
        assert_eq!(t.content(), Some("bbbbbb"));

        // Test extract public key
        let t: Tag = Tag::parse(&[
            "custom-p",
            "f86c44a2de95d9149b51c6a29afeabba264c18e2fa7c49de93424a0c56947785",
        ])
        .unwrap();
        assert_eq!(
            t.content(),
            Some("f86c44a2de95d9149b51c6a29afeabba264c18e2fa7c49de93424a0c56947785")
        );

        // Test extract event ID
        let t: Tag = Tag::parse(&[
            "custom-e",
            "2be17aa3031bdcb006f0fce80c146dea9c1c0268b0af2398bb673365c6444d45",
        ])
        .unwrap();
        assert_eq!(
            t.content(),
            Some("2be17aa3031bdcb006f0fce80c146dea9c1c0268b0af2398bb673365c6444d45")
        );
    }

    #[test]
    fn test_deserialize_tag_from_event() {
        // Got this fresh off the wire
        let event: &str = r#"{"id":"2be17aa3031bdcb006f0fce80c146dea9c1c0268b0af2398bb673365c6444d45","pubkey":"f86c44a2de95d9149b51c6a29afeabba264c18e2fa7c49de93424a0c56947785","created_at":1640839235,"kind":4,"tags":[["p","13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"]],"content":"uRuvYr585B80L6rSJiHocw==?iv=oh6LVqdsYYol3JfFnXTbPA==","sig":"a5d9290ef9659083c490b303eb7ee41356d8778ff19f2f91776c8dc4443388a64ffcf336e61af4c25c05ac3ae952d1ced889ed655b67790891222aaa15b99fdd"}"#;
        let event = Event::from_json(event).unwrap();
        let tag = event.tags().first().unwrap();

        assert_eq!(
            tag,
            &Tag::public_key(
                PublicKey::from_hex(
                    "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
                )
                .unwrap()
            )
        );
    }

    #[test]
    fn test_serialize_tag_to_event() {
        let public_key =
            PublicKey::from_hex("68d81165918100b7da43fc28f7d1fc12554466e1115886b9e7bb326f65ec4272")
                .unwrap();
        let event = Event::new(
            EventId::from_hex("378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7")
                .unwrap(),
            PublicKey::from_hex("79dff8f82963424e0bb02708a22e44b4980893e3a4be0fa3cb60a43b946764e3").unwrap(),
            Timestamp::from(1671739153),
            Kind::EncryptedDirectMessage,
            [Tag::public_key(public_key)],
            "8y4MRYrb4ztvXO2NmsHvUA==?iv=MplZo7oSdPfH/vdMC8Hmwg==",
            Signature::from_str("fd0954de564cae9923c2d8ee9ab2bf35bc19757f8e328a978958a2fcc950eaba0754148a203adec29b7b64080d0cf5a32bebedd768ea6eb421a6b751bb4584a8").unwrap()
        );

        let event_json: &str = r#"{"id":"378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7","pubkey":"79dff8f82963424e0bb02708a22e44b4980893e3a4be0fa3cb60a43b946764e3","created_at":1671739153,"kind":4,"tags":[["p","68d81165918100b7da43fc28f7d1fc12554466e1115886b9e7bb326f65ec4272"]],"content":"8y4MRYrb4ztvXO2NmsHvUA==?iv=MplZo7oSdPfH/vdMC8Hmwg==","sig":"fd0954de564cae9923c2d8ee9ab2bf35bc19757f8e328a978958a2fcc950eaba0754148a203adec29b7b64080d0cf5a32bebedd768ea6eb421a6b751bb4584a8"}"#;

        assert_eq!(&event.as_json(), event_json);
    }

    #[test]
    fn test_tag_as_vec() {
        assert_eq!(
            vec!["-"],
            Tag::from_standardized_without_cell(TagStandard::Protected).to_vec()
        );

        assert_eq!(vec!["alt", "something"], Tag::alt("something").to_vec());

        assert_eq!(
            vec!["content-warning"],
            Tag::from_standardized_without_cell(TagStandard::ContentWarning { reason: None })
                .to_vec()
        );

        assert_eq!(
            vec![
                "p",
                "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
            ],
            Tag::public_key(
                PublicKey::from_str(
                    "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
                )
                .unwrap()
            )
            .to_vec()
        );

        assert_eq!(
            vec![
                "e",
                "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7"
            ],
            Tag::event(
                EventId::from_hex(
                    "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7"
                )
                .unwrap()
            )
            .to_vec()
        );

        assert_eq!(
            vec!["expiration", "1600000000"],
            Tag::expiration(Timestamp::from(1600000000)).to_vec()
        );

        assert_eq!(
            vec!["content-warning", "reason"],
            Tag::from_standardized_without_cell(TagStandard::ContentWarning {
                reason: Some(String::from("reason"))
            })
            .to_vec()
        );

        assert_eq!(
            vec!["subject", "textnote with subject"],
            Tag::from_standardized_without_cell(TagStandard::Subject(String::from(
                "textnote with subject"
            )))
            .to_vec()
        );

        assert_eq!(
            vec!["client", "rust-nostr"],
            Tag::custom(TagKind::Custom(Cow::Borrowed("client")), ["rust-nostr"]).to_vec()
        );

        assert_eq!(vec!["d", "test"], Tag::identifier("test").to_vec());

        assert_eq!(
            vec![
                "p",
                "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d",
                "wss://relay.damus.io"
            ],
            Tag::from_standardized_without_cell(TagStandard::PublicKey {
                public_key: PublicKey::from_str(
                    "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
                )
                .unwrap(),
                relay_url: Some(UncheckedUrl::from("wss://relay.damus.io")),
                alias: None,
                uppercase: false,
            })
            .to_vec()
        );

        assert_eq!(
            vec![
                "e",
                "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7",
                ""
            ],
            Tag::from_standardized_without_cell(TagStandard::Event {
                event_id: EventId::from_hex(
                    "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7"
                )
                .unwrap(),
                relay_url: Some(UncheckedUrl::empty()),
                marker: None,
                public_key: None,
            })
            .to_vec()
        );

        assert_eq!(
            vec![
                "e",
                "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7",
                "wss://relay.damus.io"
            ],
            Tag::from_standardized_without_cell(TagStandard::Event {
                event_id: EventId::from_hex(
                    "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7"
                )
                .unwrap(),
                relay_url: Some(UncheckedUrl::from("wss://relay.damus.io")),
                marker: None,
                public_key: None,
            })
            .to_vec()
        );

        assert_eq!(
            vec![
                "p",
                "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d",
                "spam"
            ],
            Tag::public_key_report(
                PublicKey::from_str(
                    "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
                )
                .unwrap(),
                Report::Spam
            )
            .to_vec()
        );

        assert_eq!(
            vec![
                "e",
                "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7",
                "nudity"
            ],
            Tag::event_report(
                EventId::from_hex(
                    "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7"
                )
                .unwrap(),
                Report::Nudity,
            )
            .to_vec()
        );

        assert_eq!(vec!["nonce", "1", "20"], Tag::pow(1, 20).to_vec());

        assert_eq!(
            vec![
                "a",
                "30023:a695f6b60119d9521934a691347d9f78e8770b56da16bb255ee286ddf9fda919:ipsum"
            ],
            Tag::coordinate(
                Coordinate::new(
                    Kind::LongFormTextNote,
                    PublicKey::from_str(
                        "a695f6b60119d9521934a691347d9f78e8770b56da16bb255ee286ddf9fda919"
                    )
                    .unwrap()
                )
                .identifier("ipsum"),
            )
            .to_vec()
        );

        assert_eq!(
            vec![
                "a",
                "30023:a695f6b60119d9521934a691347d9f78e8770b56da16bb255ee286ddf9fda919:ipsum",
                "wss://relay.nostr.org"
            ],
            Tag::from_standardized_without_cell(TagStandard::Coordinate {
                coordinate: Coordinate::new(
                    Kind::LongFormTextNote,
                    PublicKey::from_str(
                        "a695f6b60119d9521934a691347d9f78e8770b56da16bb255ee286ddf9fda919"
                    )
                    .unwrap()
                )
                .identifier("ipsum"),
                relay_url: Some(UncheckedUrl::from_str("wss://relay.nostr.org").unwrap())
            })
            .to_vec()
        );

        assert_eq!(
            vec![
                "p",
                "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d",
                "wss://relay.damus.io",
                "Speaker",
            ],
            Tag::from_standardized_without_cell(TagStandard::PublicKeyLiveEvent {
                public_key: PublicKey::from_str(
                    "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
                )
                .unwrap(),
                relay_url: Some(UncheckedUrl::from("wss://relay.damus.io")),
                marker: LiveEventMarker::Speaker,
                proof: None
            })
            .to_vec()
        );

        assert_eq!(
            vec![
                "p",
                "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d",
                "",
                "Participant",
            ],
            Tag::from_standardized_without_cell(TagStandard::PublicKeyLiveEvent {
                public_key: PublicKey::from_str(
                    "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
                )
                .unwrap(),
                relay_url: None,
                marker: LiveEventMarker::Participant,
                proof: None
            })
            .to_vec()
        );

        assert_eq!(
            vec![
                "p",
                "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d",
                "wss://relay.damus.io",
                "alias",
            ],
            Tag::from_standardized_without_cell(TagStandard::PublicKey {
                public_key: PublicKey::from_str(
                    "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
                )
                .unwrap(),
                relay_url: Some(UncheckedUrl::from("wss://relay.damus.io")),
                alias: Some(String::from("alias")),
                uppercase: false,
            })
            .to_vec()
        );

        assert_eq!(
            vec![
                "e",
                "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7",
                "",
                "reply"
            ],
            Tag::from_standardized_without_cell(TagStandard::Event {
                event_id: EventId::from_hex(
                    "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7"
                )
                .unwrap(),
                relay_url: None,
                marker: Some(Marker::Reply),
                public_key: None,
            })
            .to_vec()
        );

        assert_eq!(
            vec![
                "e",
                "0000000000000000000000000000000000000000000000000000000000000001",
                "",
                "",
                "0000000000000000000000000000000000000000000000000000000000000001",
            ],
            TagStandard::Event {
                event_id: EventId::from_hex(
                    "0000000000000000000000000000000000000000000000000000000000000001"
                )
                .unwrap(),
                relay_url: None,
                marker: None,
                public_key: Some(
                    PublicKey::parse(
                        "0000000000000000000000000000000000000000000000000000000000000001"
                    )
                    .unwrap()
                ),
            }
            .to_vec()
        );

        assert_eq!(
            vec![
                "delegation",
                "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d",
                "kind=1",
                "fd0954de564cae9923c2d8ee9ab2bf35bc19757f8e328a978958a2fcc950eaba0754148a203adec29b7b64080d0cf5a32bebedd768ea6eb421a6b751bb4584a8",
            ],
            Tag::from_standardized_without_cell(TagStandard::Delegation { delegator: PublicKey::from_str(
                "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
            ).unwrap(), conditions: Conditions::from_str("kind=1").unwrap(), sig: Signature::from_str("fd0954de564cae9923c2d8ee9ab2bf35bc19757f8e328a978958a2fcc950eaba0754148a203adec29b7b64080d0cf5a32bebedd768ea6eb421a6b751bb4584a8").unwrap() })
                .to_vec()
        );

        assert_eq!(
            vec!["lnurl", "lnurl1dp68gurn8ghj7um5v93kketj9ehx2amn9uh8wetvdskkkmn0wahz7mrww4excup0dajx2mrv92x9xp"],
            Tag::from_standardized_without_cell(TagStandard::Lnurl(String::from("lnurl1dp68gurn8ghj7um5v93kketj9ehx2amn9uh8wetvdskkkmn0wahz7mrww4excup0dajx2mrv92x9xp"))).to_vec(),
        );

        assert_eq!(
            vec![
                "p",
                "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d",
                "wss://relay.damus.io",
                "Host",
                "a5d9290ef9659083c490b303eb7ee41356d8778ff19f2f91776c8dc4443388a64ffcf336e61af4c25c05ac3ae952d1ced889ed655b67790891222aaa15b99fdd"
            ],
            Tag::from_standardized_without_cell(TagStandard::PublicKeyLiveEvent {
                public_key: PublicKey::from_str(
                    "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
                ).unwrap(),
                relay_url: Some(UncheckedUrl::from("wss://relay.damus.io")),
                marker: LiveEventMarker::Host,
                proof: Some(Signature::from_str("a5d9290ef9659083c490b303eb7ee41356d8778ff19f2f91776c8dc4443388a64ffcf336e61af4c25c05ac3ae952d1ced889ed655b67790891222aaa15b99fdd").unwrap())
            })
                .to_vec()
        );

        assert_eq!(
            vec!["L", "#t"],
            Tag::from_standardized_without_cell(TagStandard::LabelNamespace("#t".to_string()))
                .to_vec()
        );

        assert_eq!(
            vec!["l", "IT-MI"],
            Tag::from_standardized_without_cell(TagStandard::Label(vec!["IT-MI".to_string()]))
                .to_vec()
        );

        assert_eq!(
            vec!["l", "IT-MI", "ISO-3166-2"],
            Tag::from_standardized_without_cell(TagStandard::Label(vec![
                "IT-MI".to_string(),
                "ISO-3166-2".to_string()
            ]))
            .to_vec()
        );

        assert_eq!(
            vec!["r", "wss://atlas.nostr.land/"],
            Tag::relay_metadata(Url::from_str("wss://atlas.nostr.land").unwrap(), None).to_vec()
        );

        assert_eq!(
            vec!["r", "wss://atlas.nostr.land/", "read"],
            Tag::relay_metadata(
                Url::from_str("wss://atlas.nostr.land").unwrap(),
                Some(RelayMetadata::Read)
            )
            .to_vec()
        );

        assert_eq!(
            vec!["r", "wss://atlas.nostr.land/", "write"],
            Tag::relay_metadata(
                Url::from_str("wss://atlas.nostr.land").unwrap(),
                Some(RelayMetadata::Write)
            )
            .to_vec()
        );

        assert_eq!(
            vec!["r", "wss://atlas.nostr.land", ""],
            Tag::custom(
                TagKind::SingleLetter(SingleLetterTag::lowercase(Alphabet::R)),
                ["wss://atlas.nostr.land", ""]
            )
            .to_vec()
        );

        assert_eq!(
            vec![
                "r",
                "3dbee968d1ddcdf07521e246e405e1fbb549080f1f4ef4e42526c4528f124220",
                ""
            ],
            Tag::custom(
                TagKind::SingleLetter(SingleLetterTag::lowercase(Alphabet::R)),
                [
                    "3dbee968d1ddcdf07521e246e405e1fbb549080f1f4ef4e42526c4528f124220",
                    ""
                ]
            )
            .to_vec()
        );
    }

    #[test]
    fn test_tag_parser() {
        assert_eq!(Tag::parse::<String>(&[]).unwrap_err(), Error::EmptyTag);

        assert_eq!(Tag::parse(&["-"]).unwrap(), Tag::protected());

        assert_eq!(
            Tag::parse(&["alt", "something"]).unwrap(),
            Tag::alt("something")
        );

        assert_eq!(
            Tag::parse(&["content-warning"]).unwrap(),
            Tag::from_standardized_without_cell(TagStandard::ContentWarning { reason: None })
        );

        assert_eq!(
            Tag::parse(&[
                "p",
                "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
            ])
            .unwrap(),
            Tag::public_key(
                PublicKey::from_str(
                    "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
                )
                .unwrap()
            )
        );

        assert_eq!(
            Tag::parse(&[
                "e",
                "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7"
            ])
            .unwrap(),
            Tag::event(
                EventId::from_hex(
                    "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7"
                )
                .unwrap()
            )
        );

        assert_eq!(
            Tag::parse(&["expiration", "1600000000"]).unwrap(),
            Tag::expiration(Timestamp::from(1600000000))
        );

        assert_eq!(
            Tag::parse(&["content-warning", "reason"]).unwrap(),
            Tag::from_standardized_without_cell(TagStandard::ContentWarning {
                reason: Some(String::from("reason"))
            })
        );

        assert_eq!(
            Tag::parse(&["subject", "textnote with subject"]).unwrap(),
            Tag::from_standardized_without_cell(TagStandard::Subject(String::from(
                "textnote with subject"
            )))
        );

        assert_eq!(
            Tag::parse(&["client", "nostr-sdk"]).unwrap(),
            Tag::custom(TagKind::Custom(Cow::Borrowed("client")), ["nostr-sdk"])
        );

        assert_eq!(Tag::parse(&["d", "test"]).unwrap(), Tag::identifier("test"));

        assert_eq!(
            Tag::parse(&["r", "https://example.com"]).unwrap(),
            Tag::from_standardized_without_cell(TagStandard::Reference(String::from(
                "https://example.com"
            )))
        );

        assert_eq!(
            Tag::parse(&["r", "wss://alicerelay.example.com/"]).unwrap(),
            Tag::relay_metadata(Url::from_str("wss://alicerelay.example.com").unwrap(), None)
        );

        assert_eq!(
            Tag::parse(&["i", "github:12345678", "abcdefghijklmnop"]).unwrap(),
            Tag::from_standardized_without_cell(TagStandard::ExternalIdentity(Identity {
                platform: ExternalIdentity::GitHub,
                ident: "12345678".to_string(),
                proof: "abcdefghijklmnop".to_string()
            }))
        );

        assert_eq!(
            Tag::parse(&[
                "p",
                "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d",
                "wss://relay.damus.io"
            ])
            .unwrap(),
            Tag::from_standardized_without_cell(TagStandard::PublicKey {
                public_key: PublicKey::from_str(
                    "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
                )
                .unwrap(),
                relay_url: Some(UncheckedUrl::from("wss://relay.damus.io")),
                alias: None,
                uppercase: false
            })
        );

        assert_eq!(
            Tag::parse(&[
                "e",
                "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7",
                ""
            ])
            .unwrap(),
            Tag::from_standardized_without_cell(TagStandard::Event {
                event_id: EventId::from_hex(
                    "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7"
                )
                .unwrap(),
                relay_url: Some(UncheckedUrl::empty()),
                marker: None,
                public_key: None,
            })
        );

        assert_eq!(
            Tag::parse(&[
                "e",
                "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7",
                "wss://relay.damus.io"
            ])
            .unwrap(),
            Tag::from_standardized_without_cell(TagStandard::Event {
                event_id: EventId::from_hex(
                    "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7"
                )
                .unwrap(),
                relay_url: Some(UncheckedUrl::from("wss://relay.damus.io")),
                marker: None,
                public_key: None,
            })
        );

        assert_eq!(
            Tag::parse(&[
                "p",
                "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d",
                "impersonation"
            ])
            .unwrap(),
            Tag::from_standardized_without_cell(TagStandard::PublicKeyReport(
                PublicKey::from_str(
                    "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
                )
                .unwrap(),
                Report::Impersonation
            ))
        );

        assert_eq!(
            Tag::parse(&[
                "p",
                "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d",
                "other"
            ])
            .unwrap(),
            Tag::from_standardized_without_cell(TagStandard::PublicKeyReport(
                PublicKey::from_str(
                    "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
                )
                .unwrap(),
                Report::Other
            ))
        );

        assert_eq!(
            Tag::parse(&[
                "e",
                "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7",
                "profanity"
            ])
            .unwrap(),
            Tag::from_standardized_without_cell(TagStandard::EventReport(
                EventId::from_hex(
                    "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7"
                )
                .unwrap(),
                Report::Profanity
            ))
        );

        assert_eq!(
            Tag::parse(&[
                "e",
                "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7",
                "malware"
            ])
            .unwrap(),
            Tag::from_standardized_without_cell(TagStandard::EventReport(
                EventId::from_hex(
                    "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7"
                )
                .unwrap(),
                Report::Malware
            ))
        );

        assert_eq!(Tag::parse(&["nonce", "1", "20"]).unwrap(), Tag::pow(1, 20));

        assert_eq!(
            Tag::parse(&[
                "a",
                "30023:a695f6b60119d9521934a691347d9f78e8770b56da16bb255ee286ddf9fda919:ipsum",
                "wss://relay.nostr.org"
            ])
            .unwrap(),
            Tag::from_standardized_without_cell(TagStandard::Coordinate {
                coordinate: Coordinate::new(
                    Kind::LongFormTextNote,
                    PublicKey::from_str(
                        "a695f6b60119d9521934a691347d9f78e8770b56da16bb255ee286ddf9fda919"
                    )
                    .unwrap()
                )
                .identifier("ipsum"),
                relay_url: Some(UncheckedUrl::from_str("wss://relay.nostr.org").unwrap())
            })
        );

        assert_eq!(
            Tag::parse(&["r", "wss://alicerelay.example.com/", "read"]).unwrap(),
            Tag::relay_metadata(
                Url::from_str("wss://alicerelay.example.com").unwrap(),
                Some(RelayMetadata::Read)
            )
        );

        assert_eq!(
            Tag::parse(&["r", "wss://atlas.nostr.land/"]).unwrap(),
            Tag::relay_metadata(Url::from_str("wss://atlas.nostr.land").unwrap(), None)
        );

        assert_eq!(
            Tag::parse(&["r", "wss://atlas.nostr.land/", "read"]).unwrap(),
            Tag::relay_metadata(
                Url::from_str("wss://atlas.nostr.land").unwrap(),
                Some(RelayMetadata::Read)
            )
        );

        assert_eq!(
            Tag::parse(&["r", "wss://atlas.nostr.land/", "write"]).unwrap(),
            Tag::relay_metadata(
                Url::from_str("wss://atlas.nostr.land").unwrap(),
                Some(RelayMetadata::Write)
            )
        );

        assert_eq!(
            Tag::parse(&["r", "wss://atlas.nostr.land", ""]).unwrap(),
            Tag::custom(
                TagKind::SingleLetter(SingleLetterTag::lowercase(Alphabet::R)),
                ["wss://atlas.nostr.land", ""]
            )
        );

        assert_eq!(
            Tag::parse(&[
                "r",
                "3dbee968d1ddcdf07521e246e405e1fbb549080f1f4ef4e42526c4528f124220",
                ""
            ])
            .unwrap(),
            Tag::custom(
                TagKind::SingleLetter(SingleLetterTag::lowercase(Alphabet::R)),
                [
                    "3dbee968d1ddcdf07521e246e405e1fbb549080f1f4ef4e42526c4528f124220",
                    ""
                ]
            )
        );

        assert_eq!(
            Tag::parse(&[
                "p",
                "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d",
                "wss://relay.damus.io",
                "alias",
            ])
            .unwrap(),
            Tag::from_standardized_without_cell(TagStandard::PublicKey {
                public_key: PublicKey::from_str(
                    "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
                )
                .unwrap(),
                relay_url: Some(UncheckedUrl::from("wss://relay.damus.io")),
                alias: Some(String::from("alias")),
                uppercase: false,
            })
        );

        assert_eq!(
            Tag::parse(&[
                "e",
                "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7",
                "",
                "reply"
            ])
            .unwrap(),
            Tag::from_standardized_without_cell(TagStandard::Event {
                event_id: EventId::from_hex(
                    "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7"
                )
                .unwrap(),
                relay_url: None,
                marker: Some(Marker::Reply),
                public_key: None,
            })
        );

        assert_eq!(
            Tag::parse(&[
                "e",
                "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7",
                "",
                "reply",
                "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
            ])
            .unwrap(),
            Tag::from_standardized_without_cell(TagStandard::Event {
                event_id: EventId::from_hex(
                    "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7"
                )
                .unwrap(),
                relay_url: None,
                marker: Some(Marker::Reply),
                public_key: Some(
                    PublicKey::from_hex(
                        "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
                    )
                    .unwrap()
                ),
            })
        );

        assert_eq!(
            Tag::parse(&[
                "delegation",
                "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d",
                "kind=1",
                "fd0954de564cae9923c2d8ee9ab2bf35bc19757f8e328a978958a2fcc950eaba0754148a203adec29b7b64080d0cf5a32bebedd768ea6eb421a6b751bb4584a8",
            ]).unwrap(),
            Tag::from_standardized_without_cell(TagStandard::Delegation { delegator: PublicKey::from_str(
                "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
            ).unwrap(), conditions: Conditions::from_str("kind=1").unwrap(), sig: Signature::from_str("fd0954de564cae9923c2d8ee9ab2bf35bc19757f8e328a978958a2fcc950eaba0754148a203adec29b7b64080d0cf5a32bebedd768ea6eb421a6b751bb4584a8").unwrap() })
        );

        assert_eq!(
            Tag::parse(&[
                "relays",
                "wss://relay.damus.io/",
                "wss://nostr-relay.wlvs.space/",
                "wss://nostr.fmt.wiz.biz",
                "wss//nostr.fmt.wiz.biz"
            ])
            .unwrap(),
            Tag::from_standardized_without_cell(TagStandard::Relays(vec![
                UncheckedUrl::from("wss://relay.damus.io/"),
                UncheckedUrl::from("wss://nostr-relay.wlvs.space/"),
                UncheckedUrl::from("wss://nostr.fmt.wiz.biz"),
                UncheckedUrl::from("wss//nostr.fmt.wiz.biz")
            ]))
        );

        assert_eq!(
            Tag::parse(&[
                "bolt11",
                "lnbc10u1p3unwfusp5t9r3yymhpfqculx78u027lxspgxcr2n2987mx2j55nnfs95nxnzqpp5jmrh92pfld78spqs78v9euf2385t83uvpwk9ldrlvf6ch7tpascqhp5zvkrmemgth3tufcvflmzjzfvjt023nazlhljz2n9hattj4f8jq8qxqyjw5qcqpjrzjqtc4fc44feggv7065fqe5m4ytjarg3repr5j9el35xhmtfexc42yczarjuqqfzqqqqqqqqlgqqqqqqgq9q9qxpqysgq079nkq507a5tw7xgttmj4u990j7wfggtrasah5gd4ywfr2pjcn29383tphp4t48gquelz9z78p4cq7ml3nrrphw5w6eckhjwmhezhnqpy6gyf0"]).unwrap(),
            Tag::from_standardized_without_cell(TagStandard::Bolt11("lnbc10u1p3unwfusp5t9r3yymhpfqculx78u027lxspgxcr2n2987mx2j55nnfs95nxnzqpp5jmrh92pfld78spqs78v9euf2385t83uvpwk9ldrlvf6ch7tpascqhp5zvkrmemgth3tufcvflmzjzfvjt023nazlhljz2n9hattj4f8jq8qxqyjw5qcqpjrzjqtc4fc44feggv7065fqe5m4ytjarg3repr5j9el35xhmtfexc42yczarjuqqfzqqqqqqqqlgqqqqqqgq9q9qxpqysgq079nkq507a5tw7xgttmj4u990j7wfggtrasah5gd4ywfr2pjcn29383tphp4t48gquelz9z78p4cq7ml3nrrphw5w6eckhjwmhezhnqpy6gyf0".to_string()))
        );

        assert_eq!(
            Tag::parse(&[
                "preimage",
                "5d006d2cf1e73c7148e7519a4c68adc81642ce0e25a432b2434c99f97344c15f"
            ])
            .unwrap(),
            Tag::from_standardized_without_cell(TagStandard::Preimage(
                "5d006d2cf1e73c7148e7519a4c68adc81642ce0e25a432b2434c99f97344c15f".to_string()
            ))
        );

        assert_eq!(
            Tag::parse(&[
                "description",
                "{\"pubkey\":\"32e1827635450ebb3c5a7d12c1f8e7b2b514439ac10a67eef3d9fd9c5c68e245\",\"content\":\"\",\"id\":\"d9cc14d50fcb8c27539aacf776882942c1a11ea4472f8cdec1dea82fab66279d\",\"created_at\":1674164539,\"sig\":\"77127f636577e9029276be060332ea565deaf89ff215a494ccff16ae3f757065e2bc59b2e8c113dd407917a010b3abd36c8d7ad84c0e3ab7dab3a0b0caa9835d\",\"kind\":9734,\"tags\":[[\"e\",\"3624762a1274dd9636e0c552b53086d70bc88c165bc4dc0f9e836a1eaf86c3b8\"],[\"p\",\"32e1827635450ebb3c5a7d12c1f8e7b2b514439ac10a67eef3d9fd9c5c68e245\"],[\"relays\",\"wss://relay.damus.io\",\"wss://nostr-relay.wlvs.space\",\"wss://nostr.fmt.wiz.biz\",\"wss://relay.nostr.bg\",\"wss://nostr.oxtr.dev\",\"wss://nostr.v0l.io\",\"wss://brb.io\",\"wss://nostr.bitcoiner.social\",\"ws://monad.jb55.com:8080\",\"wss://relay.snort.social\"]]}"
            ]).unwrap(),
            Tag::from_standardized_without_cell(TagStandard::Description("{\"pubkey\":\"32e1827635450ebb3c5a7d12c1f8e7b2b514439ac10a67eef3d9fd9c5c68e245\",\"content\":\"\",\"id\":\"d9cc14d50fcb8c27539aacf776882942c1a11ea4472f8cdec1dea82fab66279d\",\"created_at\":1674164539,\"sig\":\"77127f636577e9029276be060332ea565deaf89ff215a494ccff16ae3f757065e2bc59b2e8c113dd407917a010b3abd36c8d7ad84c0e3ab7dab3a0b0caa9835d\",\"kind\":9734,\"tags\":[[\"e\",\"3624762a1274dd9636e0c552b53086d70bc88c165bc4dc0f9e836a1eaf86c3b8\"],[\"p\",\"32e1827635450ebb3c5a7d12c1f8e7b2b514439ac10a67eef3d9fd9c5c68e245\"],[\"relays\",\"wss://relay.damus.io\",\"wss://nostr-relay.wlvs.space\",\"wss://nostr.fmt.wiz.biz\",\"wss://relay.nostr.bg\",\"wss://nostr.oxtr.dev\",\"wss://nostr.v0l.io\",\"wss://brb.io\",\"wss://nostr.bitcoiner.social\",\"ws://monad.jb55.com:8080\",\"wss://relay.snort.social\"]]}".to_string()))
        );

        assert_eq!(
            Tag::parse(&["amount", "10000"]).unwrap(),
            Tag::from_standardized_without_cell(TagStandard::Amount {
                millisats: 10_000,
                bolt11: None
            })
        );

        assert_eq!(
            Tag::parse(&["L", "#t"]).unwrap(),
            Tag::from_standardized_without_cell(TagStandard::LabelNamespace("#t".to_string()))
        );

        assert_eq!(
            Tag::parse(&["l", "IT-MI"]).unwrap(),
            Tag::from_standardized_without_cell(TagStandard::Label(vec!["IT-MI".to_string()]))
        );

        assert_eq!(
            Tag::parse(&["l", "IT-MI", "ISO-3166-2"]).unwrap(),
            Tag::from_standardized_without_cell(TagStandard::Label(vec![
                "IT-MI".to_string(),
                "ISO-3166-2".to_string()
            ]))
        );
    }
}

#[cfg(bench)]
mod benches {
    use test::{black_box, Bencher};

    use super::*;

    #[bench]
    pub fn get_tag_kind(bh: &mut Bencher) {
        let tag = Tag::identifier("id");
        bh.iter(|| {
            black_box(tag.kind());
        });
    }

    #[bench]
    pub fn parse_p_tag(bh: &mut Bencher) {
        let tag = &[
            "p",
            "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d",
        ];
        bh.iter(|| {
            black_box(Tag::parse(tag)).unwrap();
        });
    }

    #[bench]
    pub fn parse_p_standardized_tag(bh: &mut Bencher) {
        let tag = &[
            "p",
            "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d",
        ];
        bh.iter(|| {
            black_box(TagStandard::parse(tag)).unwrap();
        });
    }

    #[bench]
    pub fn parse_e_tag(bh: &mut Bencher) {
        let tag = &[
            "e",
            "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7",
            "wss://relay.damus.io",
        ];
        bh.iter(|| {
            black_box(Tag::parse(tag)).unwrap();
        });
    }

    #[bench]
    pub fn parse_e_standardized_tag(bh: &mut Bencher) {
        let tag = &[
            "e",
            "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7",
            "wss://relay.damus.io",
        ];
        bh.iter(|| {
            black_box(TagStandard::parse(tag)).unwrap();
        });
    }

    #[bench]
    pub fn parse_a_tag(bh: &mut Bencher) {
        let tag = &[
            "a",
            "30023:a695f6b60119d9521934a691347d9f78e8770b56da16bb255ee286ddf9fda919:ipsum",
            "wss://relay.nostr.org",
        ];
        bh.iter(|| {
            black_box(Tag::parse(tag)).unwrap();
        });
    }

    #[bench]
    pub fn parse_a_standardized_tag(bh: &mut Bencher) {
        let tag = &[
            "a",
            "30023:a695f6b60119d9521934a691347d9f78e8770b56da16bb255ee286ddf9fda919:ipsum",
            "wss://relay.nostr.org",
        ];
        bh.iter(|| {
            black_box(TagStandard::parse(tag)).unwrap();
        });
    }
}
