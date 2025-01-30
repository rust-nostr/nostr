// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Tag

use alloc::string::{String, ToString};
use alloc::vec::{IntoIter, Vec};
#[cfg(not(feature = "std"))]
use core::cell::OnceCell;
use core::cmp::Ordering;
use core::fmt;
use core::hash::{Hash, Hasher};
#[cfg(feature = "std")]
use std::sync::OnceLock as OnceCell;

use serde::de::Error as DeserializerError;
use serde::ser::SerializeSeq;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub mod cow;
mod error;
pub mod kind;
pub mod list;
pub mod standard;
pub(super) mod weak;

pub use self::cow::CowTag;
pub use self::error::Error;
pub use self::kind::TagKind;
pub use self::list::Tags;
pub use self::standard::TagStandard;
use super::id::EventId;
use crate::nips::nip01::Coordinate;
use crate::nips::nip10::Marker;
use crate::nips::nip56::Report;
use crate::nips::nip65::RelayMetadata;
use crate::types::Url;
use crate::{ImageDimensions, PublicKey, RelayUrl, SingleLetterTag, Timestamp};

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
    pub fn parse<I, S>(tag: I) -> Result<Self, Error>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        // Collect
        let tag: Vec<String> = tag.into_iter().map(|v| v.into()).collect();

        // Check if it's empty
        if tag.is_empty() {
            return Err(Error::EmptyTag);
        }

        // Construct without an empty cell
        Ok(Self::new_with_empty_cell(tag))
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
            .get_or_init(|| TagStandard::parse(self.as_slice()).ok())
            .as_ref()
    }

    /// Consume tag and get standardized tag
    #[inline]
    pub fn to_standardized(self) -> Option<TagStandard> {
        match self.standardized.into_inner() {
            Some(inner) => inner,
            None => TagStandard::parse(&self.buf).ok(),
        }
    }

    /// Get as slice of strings
    #[inline]
    pub fn as_slice(&self) -> &[String] {
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
            uppercase: false,
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
    pub fn relay_metadata(relay_url: RelayUrl, metadata: Option<RelayMetadata>) -> Self {
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

    /// Compose `["r", "<value>"]` tag
    #[inline]
    pub fn reference<T>(reference: T) -> Self
    where
        T: Into<String>,
    {
        Self::from_standardized_without_cell(TagStandard::Reference(reference.into()))
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
    pub fn image(url: Url, dimensions: Option<ImageDimensions>) -> Self {
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

impl IntoIterator for Tag {
    type Item = String;
    type IntoIter = IntoIter<Self::Item>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.buf.into_iter()
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
        let tag: Data = Data::deserialize(deserializer)?;
        Self::parse(tag).map_err(DeserializerError::custom)
    }
}

impl From<TagStandard> for Tag {
    fn from(standard: TagStandard) -> Self {
        Self::from_standardized_without_cell(standard)
    }
}

#[cfg(test)]
mod tests {
    use core::str::FromStr;

    use secp256k1::schnorr::Signature;

    use super::*;
    use crate::{Alphabet, Event, JsonUtil, Kind, Timestamp};

    #[test]
    fn test_parse_empty_tag() {
        assert_eq!(
            Tag::parse::<Vec<_>, String>(vec![]).unwrap_err(),
            Error::EmptyTag
        );
    }

    #[test]
    fn test_tag_match_standardized() {
        let tag: Tag = Tag::parse(["d", "bravery"]).unwrap();
        assert_eq!(
            tag.as_standardized(),
            Some(&TagStandard::Identifier(String::from("bravery")))
        );

        let tag: Tag = Tag::parse(["d", "test"]).unwrap();
        assert_eq!(
            tag.to_standardized(),
            Some(TagStandard::Identifier(String::from("test")))
        );
    }

    #[test]
    fn test_extract_tag_content() {
        let t: Tag = Tag::parse(["aaaaaa", "bbbbbb"]).unwrap();
        assert_eq!(t.content(), Some("bbbbbb"));

        // Test extract public key
        let t: Tag = Tag::parse([
            "custom-p",
            "f86c44a2de95d9149b51c6a29afeabba264c18e2fa7c49de93424a0c56947785",
        ])
        .unwrap();
        assert_eq!(
            t.content(),
            Some("f86c44a2de95d9149b51c6a29afeabba264c18e2fa7c49de93424a0c56947785")
        );

        // Test extract event ID
        let t: Tag = Tag::parse([
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
        let tag = event.tags.first().unwrap();

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
    fn test_tag_custom() {
        assert_eq!(
            vec!["r", "wss://atlas.nostr.land", ""],
            Tag::custom(
                TagKind::SingleLetter(SingleLetterTag::lowercase(Alphabet::R)),
                ["wss://atlas.nostr.land", ""]
            )
            .to_vec()
        );

        assert_eq!(
            Tag::parse(["r", "wss://atlas.nostr.land", ""]).unwrap(),
            Tag::custom(
                TagKind::SingleLetter(SingleLetterTag::lowercase(Alphabet::R)),
                ["wss://atlas.nostr.land", ""]
            )
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

        assert_eq!(
            Tag::parse([
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
            vec!["client", "rust-nostr"],
            Tag::custom(TagKind::Client, ["rust-nostr"]).to_vec()
        );

        assert_eq!(
            Tag::parse(["client", "nostr-sdk"]).unwrap(),
            Tag::custom(TagKind::Client, ["nostr-sdk"])
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
        let tag = [
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
        let tag = [
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
        let tag = [
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
