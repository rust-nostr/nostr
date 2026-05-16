// Copyright (c) 2026 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP-66: Relay Discovery and Liveness Monitoring
//!
//! <https://github.com/nostr-protocol/nips/blob/master/66.md>

use alloc::borrow::ToOwned;
use alloc::string::{String, ToString};
use core::convert::Infallible;
use core::fmt;
use core::num::ParseIntError;
use core::str::FromStr;
use core::time::Duration;

use super::util::take_string;
use crate::Kind;
use crate::event::tag::{Tag, TagCodec, TagCodecError, impl_tag_codec_conversions};
use crate::util::UnwrapInfallible;

const RTT_OPEN: &str = "rtt-open";
const RTT_READ: &str = "rtt-read";
const RTT_WRITE: &str = "rtt-write";
const NETWORK_TYPE: &str = "n";
const RELAY_TYPE: &str = "T";
const NIP: &str = "N";
const REQUIREMENT: &str = "R";
const TOPIC: &str = "t";
const KIND: &str = "k";
const GEOHASH: &str = "g";

/// NIP-66 error
#[derive(Debug, PartialEq)]
pub enum Error {
    /// Parse int error
    ParseInt(ParseIntError),
    /// Codec error
    Codec(TagCodecError),
}

impl core::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ParseInt(e) => e.fmt(f),
            Self::Codec(e) => e.fmt(f),
        }
    }
}

impl From<ParseIntError> for Error {
    fn from(e: ParseIntError) -> Self {
        Self::ParseInt(e)
    }
}

impl From<TagCodecError> for Error {
    fn from(e: TagCodecError) -> Self {
        Self::Codec(e)
    }
}

/// Standardized NIP-66 tags
///
/// <https://github.com/nostr-protocol/nips/blob/master/66.md>
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Nip66Tag {
    /// Relay's open round-trip time
    RttOpen(Duration),
    /// Relay's read round-trip time
    RttRead(Duration),
    /// Relay's write round-trip time
    RttWrite(Duration),
    /// Relay's network type
    NetworkType(NetworkType),
    /// Relay type
    RelayType(RelayType),
    /// NIP supported by relay
    Nip(String),
    /// Relay requirement per NIP-11's limitations
    Requirement {
        /// Relay requirement
        requirement: Requirement,
        /// Required or not
        is_required: bool,
    },
    /// Topic associated with relay
    Topic(String),
    /// Accepted or unaccepted kind
    Kind {
        /// Event kind
        kind: Kind,
        /// Accepted by relay
        is_accepted: bool,
    },
    /// NIP-52 geohash
    Geohash(String),
}

impl TagCodec for Nip66Tag {
    type Error = Error;

    fn parse<I, S>(tag: I) -> Result<Self, Self::Error>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut iter = tag.into_iter();
        let kind: S = iter.next().ok_or(TagCodecError::missing_tag_kind())?;
        match kind.as_ref() {
            RTT_OPEN => Ok(Self::RttOpen(parse_time(iter, RTT_OPEN)?)),
            RTT_READ => Ok(Self::RttRead(parse_time(iter, RTT_READ)?)),
            RTT_WRITE => Ok(Self::RttWrite(parse_time(iter, RTT_WRITE)?)),
            NETWORK_TYPE => {
                let network_type = take_string(&mut iter, "network type")?
                    .parse()
                    .unwrap_infallible();
                Ok(Self::NetworkType(network_type))
            }
            RELAY_TYPE => {
                let relay_type = take_string(&mut iter, "relay type")?
                    .parse()
                    .unwrap_infallible();
                Ok(Self::RelayType(relay_type))
            }
            NIP => Ok(Self::Nip(take_string(&mut iter, "NIP")?)),
            REQUIREMENT => {
                let value = take_string(&mut iter, "requirement")?;
                let BoolTag { value, yes } = BoolTag::parse(&value);
                Ok(Self::Requirement {
                    requirement: value.parse().unwrap_infallible(),
                    is_required: yes,
                })
            }
            TOPIC => Ok(Self::Topic(take_string(&mut iter, "topic")?)),
            KIND => {
                let value = take_string(&mut iter, "kind")?;
                let BoolTag { value, yes } = BoolTag::parse(&value);
                Ok(Self::Kind {
                    kind: value.parse().map_err(Error::ParseInt)?,
                    is_accepted: yes,
                })
            }
            GEOHASH => Ok(Self::Geohash(take_string(&mut iter, "geohash")?)),
            _ => Err(TagCodecError::Unknown.into()),
        }
    }

    fn to_tag(&self) -> Tag {
        match self {
            Self::RttOpen(time) => {
                Tag::new(vec![RTT_OPEN.to_owned(), time.as_millis().to_string()])
            }
            Self::RttRead(time) => {
                Tag::new(vec![RTT_READ.to_owned(), time.as_millis().to_string()])
            }
            Self::RttWrite(time) => {
                Tag::new(vec![RTT_WRITE.to_owned(), time.as_millis().to_string()])
            }
            Self::NetworkType(network_type) => {
                Tag::new(vec![NETWORK_TYPE.to_owned(), network_type.to_string()])
            }
            Self::RelayType(relay_type) => {
                Tag::new(vec![RELAY_TYPE.to_owned(), relay_type.to_string()])
            }
            Self::Nip(nip) => Tag::new(vec![String::from(NIP), nip.to_owned()]),
            Self::Requirement {
                requirement,
                is_required,
            } => Tag::new(vec![
                REQUIREMENT.to_owned(),
                BoolTag::to_string(requirement.as_str(), *is_required),
            ]),
            Self::Topic(topic) => Tag::new(vec![TOPIC.to_owned(), topic.to_owned()]),
            Self::Kind { kind, is_accepted } => Tag::new(vec![
                KIND.to_owned(),
                BoolTag::to_string(kind.as_u16(), *is_accepted),
            ]),
            Self::Geohash(geohash) => Tag::new(vec![GEOHASH.to_owned(), geohash.to_owned()]),
        }
    }
}

/// Network type
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NetworkType {
    /// Clearnet
    Clearnet,
    /// Tor
    Tor,
    /// I2P
    I2p,
    /// Loki
    Loki,
    /// Other
    Other(String),
}

impl FromStr for NetworkType {
    type Err = Infallible;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(match value {
            "clearnet" => Self::Clearnet,
            "tor" => Self::Tor,
            "i2p" => Self::I2p,
            "loki" => Self::Loki,
            _ => Self::Other(value.to_owned()),
        })
    }
}

impl fmt::Display for NetworkType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl NetworkType {
    /// Serialize as `&str`
    pub fn as_str(&self) -> &str {
        match self {
            Self::Clearnet => "clearnet",
            Self::Tor => "tor",
            Self::I2p => "i2p",
            Self::Loki => "loki",
            Self::Other(value) => value,
        }
    }
}

/// Relay type
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RelayType {
    /// Relays that store all the public content of a user in a way that anyone can download
    PublicOutbox,
    /// Relays that accept any event from anyone as long as they p-tag
    /// one of the subscribers or the public in general
    PublicInbox,
    /// Relays that accept any event from anyone as long as they tag one of the
    /// subscribers or the public in general, however only the tagged individual can
    /// download their tagged events
    PrivateStorage,
    /// Relays that accept events from an author and make sure only the author can download them
    PrivateInbox,
    /// Relays that implement NIP-50 and can help find content
    Search,
    /// Relays that only track kind 0 and kind 10002 to help find people and distribute content
    Directory,
    /// Relays whose read or write access is closed to members of a NIP-29 group or NIP-71 community
    Community,
    /// Relays that return events in their own algorithm in any order they prefer
    Algo,
    /// Relays that serve as archival nodes for the network
    Archival,
    /// Private Storage relays that take priority given their closer proximity
    /// (in ping latency) to the Client
    LocalCache,
    /// Storage relays for NIP-95 content and other types of binary content
    BlobRelays,
    /// Re-broadcast content to other relays (Blastr)
    Broadcast,
    /// Aggregator proxy that connects to multiple relays while sustaining
    /// only one connection to the Client (bostr)
    Proxy,
    /// Relays that store events that are not verifiable (like Decrypted NIP-17 DMs)
    Trusted,
    /// Ephemeral relays that Push to the receiver any event received by them
    Push,
    /// Catch-all variant for relay types not covered by the standard categories
    Other(String),
}

impl FromStr for RelayType {
    type Err = Infallible;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(match value {
            "PublicOutbox" => Self::PublicOutbox,
            "PublicInbox" => Self::PublicInbox,
            "PrivateStorage" => Self::PrivateStorage,
            "PrivateInbox" => Self::PrivateInbox,
            "Search" => Self::Search,
            "Directory" => Self::Directory,
            "Community" => Self::Community,
            "Algo" => Self::Algo,
            "Archival" => Self::Archival,
            "LocalCache" => Self::LocalCache,
            "BlobRelays" => Self::BlobRelays,
            "Broadcast" => Self::Broadcast,
            "Proxy" => Self::Proxy,
            "Trusted" => Self::Trusted,
            "Push" => Self::Push,
            _ => Self::Other(value.to_owned()),
        })
    }
}

impl fmt::Display for RelayType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl RelayType {
    /// Serialize as `&str`
    pub fn as_str(&self) -> &str {
        match self {
            Self::PublicOutbox => "PublicOutbox",
            Self::PublicInbox => "PublicInbox",
            Self::PrivateStorage => "PrivateStorage",
            Self::PrivateInbox => "PrivateInbox",
            Self::Search => "Search",
            Self::Directory => "Directory",
            Self::Community => "Community",
            Self::Algo => "Algo",
            Self::Archival => "Archival",
            Self::LocalCache => "LocalCache",
            Self::BlobRelays => "BlobRelays",
            Self::Broadcast => "Broadcast",
            Self::Proxy => "Proxy",
            Self::Trusted => "Trusted",
            Self::Push => "Push",
            Self::Other(value) => value,
        }
    }
}

/// Relay requirement
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Requirement {
    /// NIP-42 authentication
    Auth,
    /// Writes
    Writes,
    /// NIP-13 PoW
    Pow,
    /// Payment
    Payment,
    /// Other relay requirement
    Other(String),
}

impl FromStr for Requirement {
    type Err = Infallible;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(match value {
            "auth" => Self::Auth,
            "writes" => Self::Writes,
            "pow" => Self::Pow,
            "payment" => Self::Payment,
            _ => Self::Other(value.to_owned()),
        })
    }
}

impl fmt::Display for Requirement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Requirement {
    /// Serialize as `&str`
    pub fn as_str(&self) -> &str {
        match self {
            Self::Auth => "auth",
            Self::Writes => "writes",
            Self::Pow => "pow",
            Self::Payment => "payment",
            Self::Other(value) => value,
        }
    }
}

fn parse_time<I, S>(mut iter: I, tag: &'static str) -> Result<Duration, Error>
where
    I: Iterator<Item = S>,
    S: AsRef<str>,
{
    let time = iter
        .next()
        .ok_or(TagCodecError::Missing(tag))?
        .as_ref()
        .parse::<u64>()
        .map_err(Error::ParseInt)?;
    Ok(Duration::from_millis(time))
}

struct BoolTag<'a> {
    value: &'a str,
    yes: bool,
}

impl<'a> BoolTag<'a> {
    const NEGATION: &'static str = "!";

    fn parse(raw_value: &'a str) -> Self {
        let (value, yes) = raw_value
            .split_once(Self::NEGATION)
            .map(|(_, r)| (r, false))
            .unwrap_or_else(|| (raw_value, true));
        Self { value, yes }
    }

    fn to_string<T: fmt::Display>(value: T, yes: bool) -> String {
        format!("{}{value}", if yes { "" } else { Self::NEGATION })
    }
}

impl_tag_codec_conversions!(Nip66Tag);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standardized_rtt_open_tag() {
        let tag = ["rtt-open", "234"];
        let parsed = Nip66Tag::parse(tag).unwrap();

        assert_eq!(parsed, Nip66Tag::RttOpen(Duration::from_millis(234)));
        assert_eq!(parsed.to_tag(), Tag::parse(tag).unwrap());

        let err = Nip66Tag::parse(["rtt-open"]).unwrap_err();
        assert_eq!(err, Error::Codec(TagCodecError::Missing("rtt-open")));
    }

    #[test]
    fn test_standardized_rtt_read_tag() {
        let tag = ["rtt-read", "234"];
        let parsed = Nip66Tag::parse(tag).unwrap();

        assert_eq!(parsed, Nip66Tag::RttRead(Duration::from_millis(234)));
        assert_eq!(parsed.to_tag(), Tag::parse(tag).unwrap());

        let err = Nip66Tag::parse(["rtt-read"]).unwrap_err();
        assert_eq!(err, Error::Codec(TagCodecError::Missing("rtt-read")));
    }

    #[test]
    fn test_standardized_rtt_write_tag() {
        let tag = ["rtt-write", "234"];
        let parsed = Nip66Tag::parse(tag).unwrap();

        assert_eq!(parsed, Nip66Tag::RttWrite(Duration::from_millis(234)));
        assert_eq!(parsed.to_tag(), Tag::parse(tag).unwrap());

        let err = Nip66Tag::parse(["rtt-write"]).unwrap_err();
        assert_eq!(err, Error::Codec(TagCodecError::Missing("rtt-write")));
    }

    #[test]
    fn test_standardized_network_type_tag() {
        let tag = ["n", "clearnet"];
        let parsed = Nip66Tag::parse(tag).unwrap();
        assert_eq!(parsed, Nip66Tag::NetworkType(NetworkType::Clearnet));
        assert_eq!(parsed.to_tag(), Tag::parse(tag).unwrap());

        let tag = ["n", "fips"];
        let parsed = Nip66Tag::parse(tag).unwrap();
        assert_eq!(
            parsed,
            Nip66Tag::NetworkType(NetworkType::Other("fips".to_owned()))
        );
        assert_eq!(parsed.to_tag(), Tag::parse(tag).unwrap());

        let err = Nip66Tag::parse(["n"]).unwrap_err();
        assert_eq!(err, Error::Codec(TagCodecError::Missing("network type")));
    }

    #[test]
    fn test_standardized_relay_type_tag() {
        let tag = ["T", "PrivateInbox"];
        let parsed = Nip66Tag::parse(tag).unwrap();

        assert_eq!(
            parsed,
            Nip66Tag::RelayType("PrivateInbox".parse::<RelayType>().unwrap())
        );
        assert_eq!(parsed.to_tag(), Tag::parse(tag).unwrap());

        let err = Nip66Tag::parse(["T"]).unwrap_err();
        assert_eq!(err, Error::Codec(TagCodecError::Missing("relay type")));
    }

    #[test]
    fn test_standardized_n_tag() {
        let tag = ["N", "66"];
        let parsed = Nip66Tag::parse(tag).unwrap();

        assert_eq!(parsed, Nip66Tag::Nip("66".to_owned()));
        assert_eq!(parsed.to_tag(), Tag::parse(tag).unwrap());

        let err = Nip66Tag::parse(["N"]).unwrap_err();
        assert_eq!(err, Error::Codec(TagCodecError::Missing("NIP")));
    }

    #[test]
    fn test_standardized_r_tag() {
        let tag = ["R", "!payment"];
        let parsed = Nip66Tag::parse(tag).unwrap();
        assert_eq!(
            parsed,
            Nip66Tag::Requirement {
                requirement: Requirement::Payment,
                is_required: false
            }
        );
        assert_eq!(parsed.to_tag(), Tag::parse(tag).unwrap());

        let tag = ["R", "auth"];
        let parsed = Nip66Tag::parse(tag).unwrap();
        assert_eq!(
            parsed,
            Nip66Tag::Requirement {
                requirement: Requirement::Auth,
                is_required: true
            }
        );
        assert_eq!(parsed.to_tag(), Tag::parse(tag).unwrap());

        let tag = ["R", "!unknown"];
        let parsed = Nip66Tag::parse(tag).unwrap();
        assert_eq!(
            parsed,
            Nip66Tag::Requirement {
                requirement: Requirement::Other("unknown".to_owned()),
                is_required: false
            }
        );
        assert_eq!(parsed.to_tag(), Tag::parse(tag).unwrap());

        let err = Nip66Tag::parse(["R"]).unwrap_err();
        assert_eq!(err, Error::Codec(TagCodecError::Missing("requirement")));
    }

    #[test]
    fn test_standardized_t_tag() {
        let tag = ["t", "nsfw"];
        let parsed = Nip66Tag::parse(tag).unwrap();

        assert_eq!(parsed, Nip66Tag::Topic("nsfw".to_owned()));
        assert_eq!(parsed.to_tag(), Tag::parse(tag).unwrap());

        let err = Nip66Tag::parse(["t"]).unwrap_err();
        assert_eq!(err, Error::Codec(TagCodecError::Missing("topic")));
    }

    #[test]
    fn test_standardized_k_tag() {
        let tag = ["k", "!1"];
        let parsed = Nip66Tag::parse(tag).unwrap();
        assert_eq!(
            parsed,
            Nip66Tag::Kind {
                kind: Kind::TextNote,
                is_accepted: false
            }
        );
        assert_eq!(parsed.to_tag(), Tag::parse(tag).unwrap());

        let tag = ["k", "1"];
        let parsed = Nip66Tag::parse(tag).unwrap();
        assert_eq!(
            parsed,
            Nip66Tag::Kind {
                kind: Kind::TextNote,
                is_accepted: true
            }
        );
        assert_eq!(parsed.to_tag(), Tag::parse(tag).unwrap());

        let err = Nip66Tag::parse(["k"]).unwrap_err();
        assert_eq!(err, Error::Codec(TagCodecError::Missing("kind")));
    }

    #[test]
    fn test_standardized_g_tag() {
        let tag = ["g", "ww8p1r4t8"];
        let parsed = Nip66Tag::parse(tag).unwrap();

        assert_eq!(parsed, Nip66Tag::Geohash("ww8p1r4t8".to_owned()));
        assert_eq!(parsed.to_tag(), Tag::parse(tag).unwrap());

        let err = Nip66Tag::parse(["g"]).unwrap_err();
        assert_eq!(err, Error::Codec(TagCodecError::Missing("geohash")));
    }
}
