// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Standardized tags

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::str::FromStr;

use hashes::sha1::Hash as Sha1Hash;
use hashes::sha256::Hash as Sha256Hash;
use secp256k1::schnorr::Signature;

use super::{Error, TagKind};
use crate::event::id::EventId;
use crate::nips::nip01::Coordinate;
use crate::nips::nip10::Marker;
use crate::nips::nip26::Conditions;
use crate::nips::nip34::EUC;
use crate::nips::nip39::Identity;
use crate::nips::nip48::Protocol;
use crate::nips::nip53::{LiveEventMarker, LiveEventStatus};
use crate::nips::nip56::Report;
use crate::nips::nip65::RelayMetadata;
use crate::nips::nip73::ExternalContentId;
use crate::nips::nip90::DataVendingMachineStatus;
#[cfg(feature = "nip98")]
use crate::nips::nip98::HttpMethod;
use crate::types::{RelayUrl, Url};
use crate::{
    Alphabet, Event, ImageDimensions, JsonUtil, Kind, PublicKey, SingleLetterTag, Timestamp,
};

const ALL_RELAYS: &str = "ALL_RELAYS";

/// Standardized tag
#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TagStandard {
    /// Event
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md> and <https://github.com/nostr-protocol/nips/blob/master/10.md>
    Event {
        event_id: EventId,
        relay_url: Option<RelayUrl>,
        marker: Option<Marker>,
        /// Should be the public key of the author of the referenced event
        public_key: Option<PublicKey>,
        /// Whether the tag is an uppercase or not
        uppercase: bool,
    },
    /// Quote
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/18.md>
    Quote {
        event_id: EventId,
        relay_url: Option<RelayUrl>,
        /// Should be the public key of the author of the referenced event
        public_key: Option<PublicKey>,
    },
    /// Report event
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/56.md>
    EventReport(EventId, Report),
    /// Git clone ([`TagKind::Clone`] tag)
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/34.md>
    GitClone(Vec<Url>),
    /// Git commit
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/34.md>
    GitCommit(Sha1Hash),
    /// Git earliest unique commit ID
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/34.md>
    GitEarliestUniqueCommitId(Sha1Hash),
    /// Git repo maintainers
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/34.md>
    GitMaintainers(Vec<PublicKey>),
    /// Public Key
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    PublicKey {
        public_key: PublicKey,
        relay_url: Option<RelayUrl>,
        alias: Option<String>,
        /// Whether the tag is an uppercase or not
        uppercase: bool,
    },
    /// Report public key
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/56.md>
    PublicKeyReport(PublicKey, Report),
    PublicKeyLiveEvent {
        public_key: PublicKey,
        relay_url: Option<RelayUrl>,
        marker: LiveEventMarker,
        proof: Option<Signature>,
    },
    Reference(String),
    /// Relay Metadata
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/65.md>
    RelayMetadata {
        relay_url: RelayUrl,
        metadata: Option<RelayMetadata>,
    },
    Hashtag(String),
    Geohash(String),
    Identifier(String),
    /// External Content ID
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/73.md>
    ExternalContent {
        content: ExternalContentId,
        /// Optional URL hint to redirect people to a website if the client isn't opinionated about how to interpret the id.
        hint: Option<Url>,
        /// Whether the tag is an uppercase or not
        uppercase: bool,
    },
    /// External Identity
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/39.md>
    ExternalIdentity(Identity),
    Coordinate {
        coordinate: Coordinate,
        relay_url: Option<RelayUrl>,
        /// Whether the tag is an uppercase or not
        uppercase: bool,
    },
    Kind {
        kind: Kind,
        /// Whether the tag is an uppercase or not
        uppercase: bool,
    },
    Relay(RelayUrl),
    Relays(Vec<RelayUrl>),
    /// All relays tag
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/62.md>
    AllRelays,
    /// Proof of Work
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/13.md>
    POW {
        nonce: u128,
        difficulty: u8,
    },
    /// Client
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/89.md>
    Client {
        /// Client name
        name: String,
        /// Client address and optional hint
        address: Option<(Coordinate, Option<RelayUrl>)>,
    },
    Delegation {
        delegator: PublicKey,
        conditions: Conditions,
        sig: Signature,
    },
    ContentWarning {
        reason: Option<String>,
    },
    Expiration(Timestamp),
    Subject(String),
    Challenge(String),
    Title(String),
    Image(Url, Option<ImageDimensions>),
    Thumb(Url, Option<ImageDimensions>),
    Summary(String),
    Description(String),
    Bolt11(String),
    Preimage(String),
    Amount {
        millisats: u64,
        bolt11: Option<String>,
    },
    Lnurl(String),
    Name(String),
    PublishedAt(Timestamp),
    Url(Url),
    MimeType(String),
    Aes256Gcm {
        key: String,
        iv: String,
    },
    Server(Url),
    Sha256(Sha256Hash),
    Size(usize),
    Dim(ImageDimensions),
    Magnet(String),
    Blurhash(String),
    Streaming(Url),
    Recording(Url),
    Starts(Timestamp),
    Ends(Timestamp),
    LiveEventStatus(LiveEventStatus),
    CurrentParticipants(u64),
    TotalParticipants(u64),
    AbsoluteURL(Url),
    #[cfg(feature = "nip98")]
    Method(HttpMethod),
    Payload(Sha256Hash),
    Anon {
        msg: Option<String>,
    },
    Proxy {
        id: String,
        protocol: Protocol,
    },
    Emoji {
        /// Name given for the emoji, which MUST consist of only alphanumeric characters and underscores
        shortcode: String,
        /// URL to the corresponding image file of the emoji
        url: Url,
    },
    Encrypted,
    Request(Event),
    DataVendingMachineStatus {
        status: DataVendingMachineStatus,
        extra_info: Option<String>,
    },
    /// Label namespace
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/32.md>
    LabelNamespace(String),
    /// Label
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/32.md>
    Label {
        /// Label value
        value: String,
        /// Label namespace
        namespace: Option<String>,
    },
    /// Required dependency
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/C0.md>
    Dependency(String),
    /// File extension
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/C0.md>
    Extension(String),
    /// License of the shared content
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/C0.md>
    License(String),
    /// Runtime or environment specification
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/C0.md>
    Runtime(String),
    /// Reference to the origin repository
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/C0.md>
    Repository(String),
    /// Protected event
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/70.md>
    Protected,
    /// A short human-readable plaintext summary of what that event is about
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/31.md>
    Alt(String),
    /// List of web URLs
    Web(Vec<Url>),
    Word(String),
}

impl TagStandard {
    /// Parse tag from slice of string
    #[inline]
    pub fn parse<S>(tag: &[S]) -> Result<Self, Error>
    where
        S: AsRef<str>,
    {
        let tag_kind: TagKind = match tag.first() {
            Some(kind) => TagKind::from(kind.as_ref()),
            None => return Err(Error::KindNotFound),
        };

        Self::internal_parse(tag_kind, tag)
    }

    fn internal_parse<S>(tag_kind: TagKind, tag: &[S]) -> Result<Self, Error>
    where
        S: AsRef<str>,
    {
        match tag_kind {
            TagKind::SingleLetter(single_letter) => match single_letter {
                // Parse `a` tag
                SingleLetterTag {
                    character: Alphabet::A,
                    uppercase,
                } => {
                    return parse_a_tag(tag, uppercase);
                }
                // Parse `e` tag
                SingleLetterTag {
                    character: Alphabet::E,
                    uppercase,
                } => {
                    return parse_e_tag(tag, uppercase);
                }
                // Parse `i` tag
                SingleLetterTag {
                    character: Alphabet::I,
                    uppercase,
                } => {
                    return parse_i_tag(tag, uppercase);
                }
                // Parse `l` tag
                SingleLetterTag {
                    character: Alphabet::L,
                    uppercase,
                } => {
                    return parse_l_tag(tag, uppercase);
                }
                // Parse `p` tag
                SingleLetterTag {
                    character: Alphabet::P,
                    uppercase,
                } => {
                    return parse_p_tag(tag, uppercase);
                }
                // Parse `r` tag
                SingleLetterTag {
                    character: Alphabet::R,
                    uppercase,
                } => {
                    return parse_r_tag(tag, uppercase);
                }
                // Parse `q` tag
                SingleLetterTag {
                    character: Alphabet::Q,
                    uppercase: false,
                } => {
                    return parse_q_tag(tag);
                }
                // Parse `t` tag
                SingleLetterTag {
                    character: Alphabet::T,
                    uppercase: false,
                } => {
                    return parse_t_tag(tag);
                }
                _ => (), // Covered later
            },
            TagKind::Anon => {
                return Ok(Self::Anon {
                    msg: extract_optional_string(tag, 1).map(|s| s.to_string()),
                })
            }
            TagKind::Client => return parse_client_tag(tag),
            TagKind::Clone => {
                let urls: Vec<Url> = extract_urls(tag)?;
                return Ok(Self::GitClone(urls));
            }
            TagKind::ContentWarning => {
                return Ok(Self::ContentWarning {
                    reason: extract_optional_string(tag, 1).map(|s| s.to_string()),
                })
            }
            TagKind::Delegation => return parse_delegation_tag(tag),
            TagKind::Encrypted => return Ok(Self::Encrypted),
            TagKind::Maintainers => {
                let public_keys: Vec<PublicKey> = extract_public_keys(tag)?;
                return Ok(Self::GitMaintainers(public_keys));
            }
            TagKind::Protected => return Ok(Self::Protected),
            TagKind::Relays => {
                let urls: Vec<RelayUrl> = extract_relay_urls(tag)?;
                return Ok(Self::Relays(urls));
            }
            TagKind::Web => {
                let urls: Vec<Url> = extract_urls(tag)?;
                return Ok(Self::Web(urls));
            }
            _ => (), // Covered later
        };

        let tag_len: usize = tag.len();

        if tag_len == 2 {
            let tag_1: &str = tag[1].as_ref();

            return match tag_kind {
                TagKind::SingleLetter(SingleLetterTag {
                    character: Alphabet::G,
                    uppercase: false,
                }) => Ok(Self::Geohash(tag_1.to_string())),
                TagKind::SingleLetter(SingleLetterTag {
                    character: Alphabet::D,
                    uppercase: false,
                }) => Ok(Self::Identifier(tag_1.to_string())),
                TagKind::SingleLetter(SingleLetterTag {
                    character: Alphabet::M,
                    uppercase: false,
                }) => Ok(Self::MimeType(tag_1.to_string())),
                TagKind::SingleLetter(SingleLetterTag {
                    character: Alphabet::X,
                    uppercase: false,
                }) => Ok(Self::Sha256(Sha256Hash::from_str(tag_1)?)),
                TagKind::SingleLetter(SingleLetterTag {
                    character: Alphabet::U,
                    uppercase: false,
                }) => Ok(Self::AbsoluteURL(Url::parse(tag_1)?)),
                TagKind::Dependency => Ok(Self::Dependency(tag_1.to_string())),
                TagKind::Relay => {
                    if tag_1 == ALL_RELAYS {
                        Ok(Self::AllRelays)
                    } else {
                        Ok(Self::Relay(RelayUrl::parse(tag_1)?))
                    }
                }
                TagKind::Expiration => Ok(Self::Expiration(Timestamp::from_str(tag_1)?)),
                TagKind::Extension => Ok(Self::Extension(tag_1.to_string())),
                TagKind::License => Ok(Self::License(tag_1.to_string())),
                TagKind::Runtime => Ok(Self::Runtime(tag_1.to_string())),
                TagKind::Repository => Ok(Self::Repository(tag_1.to_string())),
                TagKind::Subject => Ok(Self::Subject(tag_1.to_string())),
                TagKind::Challenge => Ok(Self::Challenge(tag_1.to_string())),
                TagKind::Commit => Ok(Self::GitCommit(Sha1Hash::from_str(tag_1)?)),
                TagKind::Title => Ok(Self::Title(tag_1.to_string())),
                TagKind::Image => Ok(Self::Image(Url::parse(tag_1)?, None)),
                TagKind::Thumb => Ok(Self::Thumb(Url::parse(tag_1)?, None)),
                TagKind::Server => Ok(Self::Server(Url::parse(tag_1)?)),
                TagKind::Summary => Ok(Self::Summary(tag_1.to_string())),
                TagKind::PublishedAt => Ok(Self::PublishedAt(Timestamp::from_str(tag_1)?)),
                TagKind::Description => Ok(Self::Description(tag_1.to_string())),
                TagKind::Bolt11 => Ok(Self::Bolt11(tag_1.to_string())),
                TagKind::Preimage => Ok(Self::Preimage(tag_1.to_string())),
                TagKind::Amount => Ok(Self::Amount {
                    millisats: tag_1.parse()?,
                    bolt11: None,
                }),
                TagKind::Lnurl => Ok(Self::Lnurl(tag_1.to_string())),
                TagKind::Name => Ok(Self::Name(tag_1.to_string())),
                TagKind::Url => Ok(Self::Url(Url::parse(tag_1)?)),
                TagKind::Magnet => Ok(Self::Magnet(tag_1.to_string())),
                TagKind::Blurhash => Ok(Self::Blurhash(tag_1.to_string())),
                TagKind::Streaming => Ok(Self::Streaming(Url::parse(tag_1)?)),
                TagKind::Recording => Ok(Self::Recording(Url::parse(tag_1)?)),
                TagKind::Starts => Ok(Self::Starts(Timestamp::from_str(tag_1)?)),
                TagKind::Ends => Ok(Self::Ends(Timestamp::from_str(tag_1)?)),
                TagKind::Status => match DataVendingMachineStatus::from_str(tag_1) {
                    Ok(status) => Ok(Self::DataVendingMachineStatus {
                        status,
                        extra_info: None,
                    }),
                    Err(_) => Ok(Self::LiveEventStatus(LiveEventStatus::from(tag_1))), /* TODO: check if unknown status error? */
                },
                TagKind::CurrentParticipants => Ok(Self::CurrentParticipants(tag_1.parse()?)),
                TagKind::TotalParticipants => Ok(Self::TotalParticipants(tag_1.parse()?)),
                #[cfg(feature = "nip98")]
                TagKind::Method => Ok(Self::Method(HttpMethod::from_str(tag_1)?)),
                TagKind::Payload => Ok(Self::Payload(Sha256Hash::from_str(tag_1)?)),
                TagKind::Request => Ok(Self::Request(Event::from_json(tag_1)?)),
                TagKind::Word => Ok(Self::Word(tag_1.to_string())),
                TagKind::Alt => Ok(Self::Alt(tag_1.to_string())),
                TagKind::Dim => Ok(Self::Dim(ImageDimensions::from_str(tag_1)?)),
                _ => Err(Error::UnknownStandardizedTag),
            };
        }

        if tag_len == 3 {
            let tag_1: &str = tag[1].as_ref();
            let tag_2: &str = tag[2].as_ref();

            return match tag_kind {
                TagKind::Nonce => Ok(Self::POW {
                    nonce: tag_1.parse()?,
                    difficulty: tag_2.parse()?,
                }),
                TagKind::Image => Ok(Self::Image(
                    Url::parse(tag_1)?,
                    Some(ImageDimensions::from_str(tag_2)?),
                )),
                TagKind::Thumb => Ok(Self::Thumb(
                    Url::parse(tag_1)?,
                    Some(ImageDimensions::from_str(tag_2)?),
                )),
                TagKind::Aes256Gcm => Ok(Self::Aes256Gcm {
                    key: tag_1.to_string(),
                    iv: tag_2.to_string(),
                }),
                TagKind::Proxy => Ok(Self::Proxy {
                    id: tag_1.to_string(),
                    protocol: Protocol::from(tag_2),
                }),
                TagKind::Emoji => Ok(Self::Emoji {
                    shortcode: tag_1.to_string(),
                    url: Url::parse(tag_2)?,
                }),
                TagKind::Status => match DataVendingMachineStatus::from_str(tag_1) {
                    Ok(status) => Ok(Self::DataVendingMachineStatus {
                        status,
                        extra_info: Some(tag_2.to_string()),
                    }),
                    Err(_) => Err(Error::UnknownStandardizedTag),
                },
                _ => Err(Error::UnknownStandardizedTag),
            };
        }

        Err(Error::UnknownStandardizedTag)
    }

    /// Compose `TagStandard::Event` without `relay_url` and `marker`
    ///
    /// JSON: `["e", "event-id"]`
    #[inline]
    pub fn event(event_id: EventId) -> Self {
        Self::Event {
            event_id,
            relay_url: None,
            marker: None,
            public_key: None,
            uppercase: false,
        }
    }

    /// Compose `TagStandard::PublicKey` without `relay_url` and `alias`
    ///
    /// JSON: `["p", "<public-key>"]`
    #[inline]
    pub fn public_key(public_key: PublicKey) -> Self {
        Self::PublicKey {
            public_key,
            relay_url: None,
            alias: None,
            uppercase: false,
        }
    }

    /// Check if tag is an event `reply`
    #[inline]
    pub fn is_reply(&self) -> bool {
        matches!(
            self,
            Self::Event {
                marker: Some(Marker::Reply),
                ..
            }
        )
    }

    /// Get tag kind
    pub fn kind(&self) -> TagKind {
        match self {
            Self::Event { uppercase, .. } => TagKind::SingleLetter(SingleLetterTag {
                character: Alphabet::E,
                uppercase: *uppercase,
            }),
            Self::Quote { .. } => TagKind::SingleLetter(SingleLetterTag {
                character: Alphabet::Q,
                uppercase: false,
            }),
            Self::EventReport(..) => TagKind::SingleLetter(SingleLetterTag::lowercase(Alphabet::E)),
            Self::GitClone(..) => TagKind::Clone,
            Self::GitCommit(..) => TagKind::Commit,
            Self::GitEarliestUniqueCommitId(..) => {
                TagKind::SingleLetter(SingleLetterTag::lowercase(Alphabet::R))
            }
            Self::GitMaintainers(..) => TagKind::Maintainers,
            Self::PublicKey { uppercase, .. } => TagKind::SingleLetter(SingleLetterTag {
                character: Alphabet::P,
                uppercase: *uppercase,
            }),
            Self::PublicKeyReport(..) | Self::PublicKeyLiveEvent { .. } => {
                TagKind::SingleLetter(SingleLetterTag {
                    character: Alphabet::P,
                    uppercase: false,
                })
            }
            Self::Reference(..) | Self::RelayMetadata { .. } => {
                TagKind::SingleLetter(SingleLetterTag {
                    character: Alphabet::R,
                    uppercase: false,
                })
            }
            Self::Hashtag(..) => TagKind::SingleLetter(SingleLetterTag {
                character: Alphabet::T,
                uppercase: false,
            }),
            Self::Geohash(..) => TagKind::SingleLetter(SingleLetterTag {
                character: Alphabet::G,
                uppercase: false,
            }),
            Self::Identifier(..) => TagKind::SingleLetter(SingleLetterTag {
                character: Alphabet::D,
                uppercase: false,
            }),
            Self::ExternalContent { uppercase, .. } => TagKind::SingleLetter(SingleLetterTag {
                character: Alphabet::I,
                uppercase: *uppercase,
            }),
            Self::ExternalIdentity(..) => TagKind::SingleLetter(SingleLetterTag {
                character: Alphabet::I,
                uppercase: false,
            }),
            Self::Coordinate { uppercase, .. } => TagKind::SingleLetter(SingleLetterTag {
                character: Alphabet::A,
                uppercase: *uppercase,
            }),
            Self::Kind { uppercase, .. } => TagKind::SingleLetter(SingleLetterTag {
                character: Alphabet::K,
                uppercase: *uppercase,
            }),
            Self::Relay(..) | Self::AllRelays => TagKind::Relay,
            Self::POW { .. } => TagKind::Nonce,
            Self::Client { .. } => TagKind::Client,
            Self::Delegation { .. } => TagKind::Delegation,
            Self::ContentWarning { .. } => TagKind::ContentWarning,
            Self::Expiration(..) => TagKind::Expiration,
            Self::Subject(..) => TagKind::Subject,
            Self::Challenge(..) => TagKind::Challenge,
            Self::Title(..) => TagKind::Title,
            Self::Image(..) => TagKind::Image,
            Self::Thumb(..) => TagKind::Thumb,
            Self::Summary(..) => TagKind::Summary,
            Self::PublishedAt(..) => TagKind::PublishedAt,
            Self::Description(..) => TagKind::Description,
            Self::Bolt11(..) => TagKind::Bolt11,
            Self::Preimage(..) => TagKind::Preimage,
            Self::Relays(..) => TagKind::Relays,
            Self::Amount { .. } => TagKind::Amount,
            Self::Name(..) => TagKind::Name,
            Self::Dependency(..) => TagKind::Dependency,
            Self::Extension(..) => TagKind::Extension,
            Self::License(..) => TagKind::License,
            Self::Runtime(..) => TagKind::Runtime,
            Self::Repository(..) => TagKind::Repository,
            Self::Lnurl(..) => TagKind::Lnurl,
            Self::Url(..) => TagKind::Url,
            Self::MimeType(..) => TagKind::SingleLetter(SingleLetterTag {
                character: Alphabet::M,
                uppercase: false,
            }),
            Self::Aes256Gcm { .. } => TagKind::Aes256Gcm,
            Self::Server(..) => TagKind::Server,
            Self::Sha256(..) => TagKind::SingleLetter(SingleLetterTag {
                character: Alphabet::X,
                uppercase: false,
            }),
            Self::Size(..) => TagKind::Size,
            Self::Dim(..) => TagKind::Dim,
            Self::Magnet(..) => TagKind::Magnet,
            Self::Blurhash(..) => TagKind::Blurhash,
            Self::Streaming(..) => TagKind::Streaming,
            Self::Recording(..) => TagKind::Recording,
            Self::Starts(..) => TagKind::Starts,
            Self::Ends(..) => TagKind::Ends,
            Self::LiveEventStatus(..) | Self::DataVendingMachineStatus { .. } => TagKind::Status,
            Self::CurrentParticipants(..) => TagKind::CurrentParticipants,
            Self::TotalParticipants(..) => TagKind::TotalParticipants,
            Self::AbsoluteURL(..) => TagKind::SingleLetter(SingleLetterTag {
                character: Alphabet::U,
                uppercase: false,
            }),
            #[cfg(feature = "nip98")]
            Self::Method(..) => TagKind::Method,
            Self::Payload(..) => TagKind::Payload,
            Self::Anon { .. } => TagKind::Anon,
            Self::Proxy { .. } => TagKind::Proxy,
            Self::Emoji { .. } => TagKind::Emoji,
            Self::Encrypted => TagKind::Encrypted,
            Self::Request(..) => TagKind::Request,
            Self::Word(..) => TagKind::Word,
            Self::LabelNamespace(..) => TagKind::SingleLetter(SingleLetterTag {
                character: Alphabet::L,
                uppercase: true,
            }),
            Self::Label { .. } => TagKind::SingleLetter(SingleLetterTag {
                character: Alphabet::L,
                uppercase: false,
            }),
            Self::Protected => TagKind::Protected,
            Self::Alt(..) => TagKind::Alt,
            Self::Web(..) => TagKind::Web,
        }
    }

    /// Consume tag and return string vector
    #[inline]
    pub fn to_vec(self) -> Vec<String> {
        self.into()
    }
}

impl From<TagStandard> for Vec<String> {
    fn from(standard: TagStandard) -> Self {
        let tag_kind: String = standard.kind().to_string();
        let tag: Vec<String> = match standard {
            TagStandard::Event {
                event_id,
                relay_url,
                marker,
                public_key,
                ..
            } => {
                // ["e", <event-id>, <relay-url>, <marker>, <pubkey>]
                // <relay-url>, <marker> and <pubkey> are optional
                // <relay-url>, if empty, may be set to "" (if there are additional fields later)
                // <marker> is optional and if present is one of "reply", "root", or "mention" (so not an empty string)

                let mut tag: Vec<String> = vec![tag_kind, event_id.to_hex()];

                // Check if <relay-url> exists or if there are additional fields after
                match (relay_url, marker.is_some() || public_key.is_some()) {
                    (Some(relay_url), ..) => tag.push(relay_url.to_string()),
                    (None, true) => tag.push(String::new()),
                    (None, false) => {}
                }

                if let Some(marker) = marker {
                    tag.push(marker.to_string());
                }

                if let Some(public_key) = public_key {
                    tag.push(public_key.to_string());
                }

                tag
            }
            TagStandard::Quote {
                event_id,
                relay_url,
                public_key,
            } => {
                let mut tag = vec![tag_kind, event_id.to_hex()];
                if let Some(relay_url) = relay_url {
                    tag.push(relay_url.to_string());
                }
                if let Some(public_key) = public_key {
                    // If <relay-url> is `None`, push an empty string
                    tag.resize_with(3, String::new);
                    tag.push(public_key.to_hex());
                }
                tag
            }
            TagStandard::PublicKey {
                public_key,
                relay_url,
                alias,
                ..
            } => {
                let mut tag = vec![tag_kind, public_key.to_string()];
                if let Some(relay_url) = relay_url {
                    tag.push(relay_url.to_string());
                }
                if let Some(alias) = alias {
                    tag.resize_with(3, String::new);
                    tag.push(alias);
                }
                tag
            }
            TagStandard::EventReport(id, report) => {
                vec![tag_kind, id.to_hex(), report.to_string()]
            }
            TagStandard::GitClone(urls) => {
                let mut tag: Vec<String> = Vec::with_capacity(1 + urls.len());
                tag.push(tag_kind);
                tag.extend(urls.into_iter().map(|url| url.to_string()));
                tag
            }
            TagStandard::GitCommit(hash) => {
                vec![tag_kind, hash.to_string()]
            }
            TagStandard::GitEarliestUniqueCommitId(commit) => {
                vec![tag_kind, commit.to_string(), EUC.to_string()]
            }
            TagStandard::GitMaintainers(public_keys) => {
                let mut tag: Vec<String> = Vec::with_capacity(1 + public_keys.len());
                tag.push(tag_kind);
                tag.extend(public_keys.into_iter().map(|val| val.to_string()));
                tag
            }
            TagStandard::PublicKeyReport(pk, report) => {
                vec![tag_kind, pk.to_string(), report.to_string()]
            }
            TagStandard::PublicKeyLiveEvent {
                public_key,
                relay_url,
                marker,
                proof,
            } => {
                let mut tag = vec![
                    tag_kind,
                    public_key.to_string(),
                    relay_url.map(|u| u.to_string()).unwrap_or_default(),
                    marker.to_string(),
                ];
                if let Some(proof) = proof {
                    tag.push(proof.to_string());
                }
                tag
            }
            TagStandard::Reference(r) => vec![tag_kind, r],
            TagStandard::RelayMetadata {
                relay_url,
                metadata,
            } => {
                let mut tag = vec![tag_kind, relay_url.to_string()];
                if let Some(metadata) = metadata {
                    tag.push(metadata.to_string());
                }
                tag
            }
            TagStandard::Hashtag(t) => vec![tag_kind, t],
            TagStandard::Geohash(g) => vec![tag_kind, g],
            TagStandard::Identifier(d) => vec![tag_kind, d],
            TagStandard::Coordinate {
                coordinate,
                relay_url,
                ..
            } => {
                let mut vec = vec![tag_kind, coordinate.to_string()];
                if let Some(relay) = relay_url {
                    vec.push(relay.to_string());
                }
                vec
            }
            TagStandard::ExternalContent { content, hint, .. } => {
                let mut tag = vec![tag_kind, content.to_string()];

                if let Some(hint) = hint {
                    tag.push(hint.to_string());
                }

                tag
            }
            TagStandard::ExternalIdentity(identity) => {
                vec![tag_kind, identity.tag_platform_identity(), identity.proof]
            }
            TagStandard::Kind { kind, .. } => vec![tag_kind, kind.to_string()],
            TagStandard::Relay(url) => vec![tag_kind, url.to_string()],
            TagStandard::AllRelays => vec![tag_kind, ALL_RELAYS.to_string()],
            TagStandard::POW { nonce, difficulty } => {
                vec![tag_kind, nonce.to_string(), difficulty.to_string()]
            }
            TagStandard::Client { name, address } => {
                let mut tag: Vec<String> = vec![tag_kind, name];

                match address {
                    Some((coordinate, Some(hint))) => {
                        tag.reserve_exact(2);
                        tag.push(coordinate.to_string());
                        tag.push(hint.to_string());
                    }
                    Some((coordinate, None)) => {
                        tag.push(coordinate.to_string());
                    }
                    _ => {}
                }

                tag
            }
            TagStandard::Delegation {
                delegator,
                conditions,
                sig,
            } => vec![
                tag_kind,
                delegator.to_string(),
                conditions.to_string(),
                sig.to_string(),
            ],
            TagStandard::ContentWarning { reason } => {
                let mut tag = vec![tag_kind];
                if let Some(reason) = reason {
                    tag.push(reason);
                }
                tag
            }
            TagStandard::Expiration(timestamp) => {
                vec![tag_kind, timestamp.to_string()]
            }
            TagStandard::Subject(sub) => vec![tag_kind, sub],
            TagStandard::Challenge(challenge) => vec![tag_kind, challenge],
            TagStandard::Title(title) => vec![tag_kind, title],
            TagStandard::Image(image, dimensions) => {
                let mut tag = vec![tag_kind, image.to_string()];
                if let Some(dim) = dimensions {
                    tag.push(dim.to_string());
                }
                tag
            }
            TagStandard::Thumb(thumb, dimensions) => {
                let mut tag = vec![tag_kind, thumb.to_string()];
                if let Some(dim) = dimensions {
                    tag.push(dim.to_string());
                }
                tag
            }
            TagStandard::Summary(summary) => vec![tag_kind, summary],
            TagStandard::PublishedAt(timestamp) => {
                vec![tag_kind, timestamp.to_string()]
            }
            TagStandard::Description(description) => {
                vec![tag_kind, description]
            }
            TagStandard::Bolt11(bolt11) => {
                vec![tag_kind, bolt11]
            }
            TagStandard::Preimage(preimage) => {
                vec![tag_kind, preimage]
            }
            TagStandard::Relays(relays) => vec![tag_kind]
                .into_iter()
                .chain(relays.iter().map(|relay| relay.to_string()))
                .collect::<Vec<_>>(),
            TagStandard::Amount { millisats, bolt11 } => {
                let mut tag = vec![tag_kind, millisats.to_string()];
                if let Some(bolt11) = bolt11 {
                    tag.push(bolt11);
                }
                tag
            }
            TagStandard::Name(name) => vec![tag_kind, name],
            TagStandard::Dependency(dep) => vec![tag_kind, dep],
            TagStandard::Extension(ext) => vec![tag_kind, ext],
            TagStandard::License(license) => vec![tag_kind, license],
            TagStandard::Runtime(runtime) => vec![tag_kind, runtime],
            TagStandard::Repository(repo) => vec![tag_kind, repo],
            TagStandard::Lnurl(lnurl) => vec![tag_kind, lnurl],
            TagStandard::Url(url) => vec![tag_kind, url.to_string()],
            TagStandard::MimeType(mime) => vec![tag_kind, mime],
            TagStandard::Aes256Gcm { key, iv } => vec![tag_kind, key, iv],
            TagStandard::Server(url) => vec![tag_kind, url.to_string()],
            TagStandard::Sha256(hash) => vec![tag_kind, hash.to_string()],
            TagStandard::Size(bytes) => vec![tag_kind, bytes.to_string()],
            TagStandard::Dim(dim) => vec![tag_kind, dim.to_string()],
            TagStandard::Magnet(uri) => vec![tag_kind, uri],
            TagStandard::Blurhash(data) => vec![tag_kind, data],
            TagStandard::Streaming(url) => vec![tag_kind, url.to_string()],
            TagStandard::Recording(url) => vec![tag_kind, url.to_string()],
            TagStandard::Starts(timestamp) => {
                vec![tag_kind, timestamp.to_string()]
            }
            TagStandard::Ends(timestamp) => {
                vec![tag_kind, timestamp.to_string()]
            }
            TagStandard::LiveEventStatus(s) => {
                vec![tag_kind, s.to_string()]
            }
            TagStandard::CurrentParticipants(num) => {
                vec![tag_kind, num.to_string()]
            }
            TagStandard::TotalParticipants(num) => {
                vec![tag_kind, num.to_string()]
            }
            TagStandard::AbsoluteURL(url) => {
                vec![tag_kind, url.to_string()]
            }
            #[cfg(feature = "nip98")]
            TagStandard::Method(method) => {
                vec![tag_kind, method.to_string()]
            }
            TagStandard::Payload(p) => vec![tag_kind, p.to_string()],
            TagStandard::Anon { msg } => {
                let mut tag = vec![tag_kind];
                if let Some(msg) = msg {
                    tag.push(msg);
                }
                tag
            }
            TagStandard::Proxy { id, protocol } => {
                vec![tag_kind, id, protocol.to_string()]
            }
            TagStandard::Emoji { shortcode, url } => {
                vec![tag_kind, shortcode, url.to_string()]
            }
            TagStandard::Encrypted => vec![tag_kind],
            TagStandard::Request(event) => vec![tag_kind, event.as_json()],
            TagStandard::DataVendingMachineStatus { status, extra_info } => {
                let mut tag = vec![tag_kind, status.to_string()];
                if let Some(extra_info) = extra_info {
                    tag.push(extra_info);
                }
                tag
            }
            TagStandard::Word(word) => vec![tag_kind, word],
            TagStandard::LabelNamespace(n) => vec![tag_kind, n],
            TagStandard::Label { value, namespace } => {
                let mut tag: Vec<String> = vec![tag_kind, value];
                if let Some(namespace) = namespace {
                    tag.push(namespace);
                }
                tag
            }
            TagStandard::Protected => vec![tag_kind],
            TagStandard::Alt(summary) => vec![tag_kind, summary],
            TagStandard::Web(urls) => {
                let mut tag: Vec<String> = Vec::with_capacity(1 + urls.len());
                tag.push(tag_kind);
                tag.extend(urls.into_iter().map(|url| url.to_string()));
                tag
            }
        };

        // Tag can't be empty, require at least 1 value
        assert!(!tag.is_empty(), "Empty tag");

        tag
    }
}

fn parse_a_tag<S>(tag: &[S], uppercase: bool) -> Result<TagStandard, Error>
where
    S: AsRef<str>,
{
    if tag.len() >= 2 {
        Ok(TagStandard::Coordinate {
            coordinate: Coordinate::from_str(tag[1].as_ref())?,
            relay_url: match tag.get(2).map(|u| u.as_ref()) {
                Some(url) if !url.is_empty() => Some(RelayUrl::parse(url)?),
                _ => None,
            },
            uppercase,
        })
    } else {
        Err(Error::UnknownStandardizedTag)
    }
}

fn parse_e_tag<S>(tag: &[S], uppercase: bool) -> Result<TagStandard, Error>
where
    S: AsRef<str>,
{
    if tag.len() < 2 {
        return Err(Error::UnknownStandardizedTag);
    }

    let event_id: EventId = EventId::from_hex(tag[1].as_ref())?;

    let tag_2: Option<&str> = tag.get(2).map(|r| r.as_ref());
    let tag_3: Option<&str> = tag.get(3).map(|r| r.as_ref());
    let tag_4: Option<&str> = tag.get(4).map(|r| r.as_ref());

    // Check if it's a report
    if let Some(tag_2) = tag_2 {
        return match Report::from_str(tag_2) {
            // Valid report
            Ok(report) if !uppercase => Ok(TagStandard::EventReport(event_id, report)),
            // Invalid report: uppercase "E" tag
            Ok(..) if uppercase => Err(Error::UnknownStandardizedTag),
            // Not a report
            _ => {
                // Check if 3rd arg is a marker or a public key
                let (marker, public_key) = match (tag_3, tag_4) {
                    (Some(marker), Some(public_key)) => {
                        let marker = if marker.is_empty() {
                            None
                        } else {
                            Some(Marker::from_str(marker)?)
                        };
                        let public_key = PublicKey::from_hex(public_key)?;
                        (marker, Some(public_key))
                    }
                    (Some(marker), None) => {
                        if marker.is_empty() {
                            (None, None)
                        } else {
                            match Marker::from_str(marker) {
                                Ok(marker) => (Some(marker), None),
                                Err(..) => {
                                    let public_key = PublicKey::from_hex(marker)?;
                                    (None, Some(public_key))
                                }
                            }
                        }
                    }
                    (None, Some(public_key)) => {
                        let public_key = PublicKey::from_hex(public_key)?;
                        (None, Some(public_key))
                    }
                    (None, None) => (None, None),
                };

                Ok(TagStandard::Event {
                    event_id,
                    relay_url: if !tag_2.is_empty() {
                        Some(RelayUrl::parse(tag_2)?)
                    } else {
                        None
                    },
                    marker,
                    public_key,
                    uppercase,
                })
            }
        };
    }

    Ok(TagStandard::Event {
        event_id,
        relay_url: None,
        marker: None,
        public_key: None,
        uppercase,
    })
}

fn parse_i_tag<S>(tag: &[S], uppercase: bool) -> Result<TagStandard, Error>
where
    S: AsRef<str>,
{
    // External Content ID (NIP73) has min 2 values
    // External Identity (NI39) has min 3 values
    if tag.len() < 2 {
        return Err(Error::UnknownStandardizedTag);
    }

    let tag_1: &str = tag[1].as_ref();
    let tag_2: Option<&str> = tag.get(2).map(|t| t.as_ref());

    // Check if External Identity (NIP39)
    if !uppercase {
        if let Some(tag_2) = tag_2 {
            if let Ok(identity) = Identity::new(tag_1, tag_2) {
                return Ok(TagStandard::ExternalIdentity(identity));
            }
        }
    }

    // Check if External Content ID (NIP73)
    if let Ok(content) = ExternalContentId::from_str(tag_1) {
        return Ok(TagStandard::ExternalContent {
            content,
            hint: match tag_2 {
                Some(url) => Some(Url::parse(url)?),
                None => None,
            },
            uppercase,
        });
    }

    Err(Error::UnknownStandardizedTag)
}

fn parse_l_tag<S>(tag: &[S], uppercase: bool) -> Result<TagStandard, Error>
where
    S: AsRef<str>,
{
    // ["L", "<namespace>"]
    if uppercase && tag.len() == 2 {
        let tag_1: &str = tag[1].as_ref();
        return Ok(TagStandard::LabelNamespace(tag_1.to_string()));
    }

    // ["l", "<label>"] or ["l", "<label>", "<namespace>"]
    if !uppercase && tag.len() >= 2 {
        let tag_1: &str = tag[1].as_ref();
        let tag_2: Option<&str> = tag.get(2).map(|t| t.as_ref());

        return Ok(TagStandard::Label {
            value: tag_1.to_string(),
            namespace: tag_2.map(|n| n.to_string()),
        });
    }

    Err(Error::UnknownStandardizedTag)
}

fn parse_p_tag<S>(tag: &[S], uppercase: bool) -> Result<TagStandard, Error>
where
    S: AsRef<str>,
{
    if tag.len() >= 2 {
        let public_key: PublicKey = PublicKey::from_hex(tag[1].as_ref())?;

        if tag.len() >= 5 && !uppercase {
            let tag_2: &str = tag[2].as_ref();
            let tag_3: &str = tag[3].as_ref();
            let tag_4: &str = tag[4].as_ref();

            return Ok(TagStandard::PublicKeyLiveEvent {
                public_key,
                relay_url: if !tag_2.is_empty() {
                    Some(RelayUrl::parse(tag_2)?)
                } else {
                    None
                },
                marker: LiveEventMarker::from_str(tag_3)?,
                proof: Signature::from_str(tag_4).ok(),
            });
        }

        if tag.len() >= 4 && !uppercase {
            let tag_2: &str = tag[2].as_ref();
            let tag_3: &str = tag[3].as_ref();

            let relay_url: Option<RelayUrl> = if !tag_2.is_empty() {
                Some(RelayUrl::parse(tag_2)?)
            } else {
                None
            };

            return match LiveEventMarker::from_str(tag_3) {
                Ok(marker) => Ok(TagStandard::PublicKeyLiveEvent {
                    public_key,
                    relay_url,
                    marker,
                    proof: None,
                }),
                Err(_) => Ok(TagStandard::PublicKey {
                    public_key,
                    relay_url,
                    alias: (!tag_3.is_empty()).then_some(tag_3.to_string()),
                    uppercase,
                }),
            };
        }

        if tag.len() >= 3 && !uppercase {
            let tag_2: &str = tag[2].as_ref();

            return if tag_2.is_empty() {
                Ok(TagStandard::PublicKey {
                    public_key,
                    relay_url: None,
                    alias: None,
                    uppercase,
                })
            } else {
                match Report::from_str(tag_2) {
                    Ok(report) => Ok(TagStandard::PublicKeyReport(public_key, report)),
                    Err(_) => Ok(TagStandard::PublicKey {
                        public_key,
                        relay_url: Some(RelayUrl::parse(tag_2)?),
                        alias: None,
                        uppercase,
                    }),
                }
            };
        }

        Ok(TagStandard::PublicKey {
            public_key,
            relay_url: None,
            alias: None,
            uppercase,
        })
    } else {
        Err(Error::UnknownStandardizedTag)
    }
}

fn parse_r_tag<S>(tag: &[S], uppercase: bool) -> Result<TagStandard, Error>
where
    S: AsRef<str>,
{
    if tag.len() >= 3 && !uppercase {
        let tag_1: &str = tag[1].as_ref();
        let tag_2: &str = tag[2].as_ref();

        return if tag_1.starts_with("ws://") || tag_1.starts_with("wss://") {
            Ok(TagStandard::RelayMetadata {
                relay_url: RelayUrl::parse(tag_1)?,
                metadata: Some(RelayMetadata::from_str(tag_2)?),
            })
        } else if tag_2 == EUC {
            // ["r", "<commit-id>", "euc"]
            let commit: Sha1Hash = Sha1Hash::from_str(tag_1)?;
            Ok(TagStandard::GitEarliestUniqueCommitId(commit))
        } else {
            Err(Error::UnknownStandardizedTag)
        };
    }

    if tag.len() >= 2 && !uppercase {
        let tag_1: &str = tag[1].as_ref();

        return if tag_1.starts_with("ws://") || tag_1.starts_with("wss://") {
            Ok(TagStandard::RelayMetadata {
                relay_url: RelayUrl::parse(tag_1)?,
                metadata: None,
            })
        } else {
            Ok(TagStandard::Reference(tag_1.to_string()))
        };
    }

    Err(Error::UnknownStandardizedTag)
}

fn parse_q_tag<S>(tag: &[S]) -> Result<TagStandard, Error>
where
    S: AsRef<str>,
{
    if tag.len() < 2 {
        return Err(Error::UnknownStandardizedTag);
    }

    let event_id: EventId = EventId::from_hex(tag[1].as_ref())?;

    let tag_2: Option<&str> = tag.get(2).map(|r| r.as_ref());
    let tag_3: Option<&str> = tag.get(3).map(|r| r.as_ref());

    let relay_url: Option<RelayUrl> = match tag_2 {
        Some(url) if !url.is_empty() => Some(RelayUrl::parse(url)?),
        _ => None,
    };

    let public_key: Option<PublicKey> = match tag_3 {
        Some(public_key) => Some(PublicKey::from_hex(public_key)?),
        None => None,
    };

    Ok(TagStandard::Quote {
        event_id,
        relay_url,
        public_key,
    })
}

fn parse_t_tag<S>(tag: &[S]) -> Result<TagStandard, Error>
where
    S: AsRef<str>,
{
    // ["t", "<hashtag>"]

    let hashtag: &str = tag.get(1).ok_or(Error::UnknownStandardizedTag)?.as_ref();

    // Not all languages have distinct uppercase and lowercase letters.
    // `char::is_uppercase` and `char::is_lowercase` will return `false` for those languages.
    // So, to verify that a hashtag is invalid (non-lowercase),
    // check if there is at least one uppercase char.
    if hashtag.chars().any(char::is_uppercase) {
        return Err(Error::UnknownStandardizedTag);
    }

    Ok(TagStandard::Hashtag(hashtag.to_string()))
}

fn parse_client_tag<S>(tag: &[S]) -> Result<TagStandard, Error>
where
    S: AsRef<str>,
{
    // Possible cases:
    // - ["client", "My Client"]
    // - ["client", "My Client", "31990:app1-pubkey:<d-identifier>"]
    // - ["client", "My Client", "31990:app1-pubkey:<d-identifier>", "wss://relay1"]

    // Require at least 2 values
    if tag.len() < 2 {
        return Err(Error::UnknownStandardizedTag);
    }

    // The client name
    let tag_1: &str = tag[1].as_ref();

    // Optionally, the coordinate and the relay hint
    let tag_2: Option<&str> = tag.get(2).map(|t| t.as_ref());
    let tag_3: Option<&str> = tag.get(3).map(|t| t.as_ref());

    // Since the address is optional,
    // don't return an error if the coordinate or relay hint parsing fails.
    let address: Option<(Coordinate, Option<RelayUrl>)> = match tag_2 {
        // Try to parse the coordinate
        Some(coordinate) => match Coordinate::parse(coordinate) {
            // Coordinate parsing success
            Ok(coordinate) => {
                let relay_url: Option<RelayUrl> = tag_3.and_then(|url| RelayUrl::parse(url).ok());
                Some((coordinate, relay_url))
            }
            // Failed to parse the coordinate
            Err(..) => None,
        },
        // Nothing to parse
        None => None,
    };

    // Construct tag
    Ok(TagStandard::Client {
        name: tag_1.to_string(),
        address,
    })
}

fn parse_delegation_tag<S>(tag: &[S]) -> Result<TagStandard, Error>
where
    S: AsRef<str>,
{
    if tag.len() == 4 {
        let tag_1: &str = tag[1].as_ref();
        let tag_2: &str = tag[2].as_ref();
        let tag_3: &str = tag[3].as_ref();

        Ok(TagStandard::Delegation {
            delegator: PublicKey::from_hex(tag_1)?,
            conditions: Conditions::from_str(tag_2)?,
            sig: Signature::from_str(tag_3)?,
        })
    } else {
        Err(Error::UnknownStandardizedTag)
    }
}

#[inline]
fn extract_optional_string<S>(tag: &[S], index: usize) -> Option<&str>
where
    S: AsRef<str>,
{
    match tag.get(index).map(|t| t.as_ref()) {
        Some(t) => (!t.is_empty()).then_some(t),
        None => None,
    }
}

fn extract_urls<S>(tag: &[S]) -> Result<Vec<Url>, Error>
where
    S: AsRef<str>,
{
    // Skip index 0 because is the tag kind
    let mut list: Vec<Url> = Vec::with_capacity(tag.len().saturating_sub(1));
    for url in tag.iter().skip(1) {
        list.push(Url::parse(url.as_ref())?);
    }
    Ok(list)
}

fn extract_relay_urls<S>(tag: &[S]) -> Result<Vec<RelayUrl>, Error>
where
    S: AsRef<str>,
{
    // Skip index 0 because is the tag kind
    let mut list: Vec<RelayUrl> = Vec::with_capacity(tag.len().saturating_sub(1));
    for url in tag.iter().skip(1) {
        list.push(RelayUrl::parse(url.as_ref())?);
    }
    Ok(list)
}

fn extract_public_keys<S>(tag: &[S]) -> Result<Vec<PublicKey>, Error>
where
    S: AsRef<str>,
{
    // Skip index 0 because is the tag kind
    let mut list: Vec<PublicKey> = Vec::with_capacity(tag.len().saturating_sub(1));
    for url in tag.iter().skip(1) {
        list.push(PublicKey::parse(url.as_ref())?);
    }
    Ok(list)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nips::nip39::ExternalIdentity;

    #[test]
    fn test_tag_standard_is_reply() {
        let tag = TagStandard::Relay(RelayUrl::parse("wss://relay.damus.io").unwrap());
        assert!(!tag.is_reply());

        let tag = TagStandard::Event {
            event_id: EventId::from_hex(
                "2be17aa3031bdcb006f0fce80c146dea9c1c0268b0af2398bb673365c6444d45",
            )
            .unwrap(),
            relay_url: None,
            marker: Some(Marker::Reply),
            public_key: None,
            uppercase: false,
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
            uppercase: false,
        };
        assert!(!tag.is_reply());
    }

    #[test]
    fn test_tag_standard_serialization() {
        assert_eq!(vec!["-"], TagStandard::Protected.to_vec());

        assert_eq!(
            vec!["alt", "something"],
            TagStandard::Alt(String::from("something")).to_vec()
        );

        assert_eq!(
            vec!["content-warning"],
            TagStandard::ContentWarning { reason: None }.to_vec()
        );

        assert_eq!(
            vec![
                "p",
                "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
            ],
            TagStandard::public_key(
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
            TagStandard::event(
                EventId::from_hex(
                    "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7"
                )
                .unwrap()
            )
            .to_vec()
        );

        assert_eq!(
            vec![
                "q",
                "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7"
            ],
            TagStandard::Quote {
                event_id: EventId::from_hex(
                    "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7"
                )
                .unwrap(),
                relay_url: None,
                public_key: None,
            }
            .to_vec()
        );

        assert_eq!(
            vec![
                "q",
                "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7",
                "wss://relay.damus.io"
            ],
            TagStandard::Quote {
                event_id: EventId::from_hex(
                    "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7"
                )
                .unwrap(),
                relay_url: Some(RelayUrl::parse("wss://relay.damus.io").unwrap()),
                public_key: None,
            }
            .to_vec()
        );

        assert_eq!(
            vec![
                "q",
                "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7",
                "",
                "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
            ],
            TagStandard::Quote {
                event_id: EventId::from_hex(
                    "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7"
                )
                .unwrap(),
                relay_url: None,
                public_key: Some(
                    PublicKey::from_str(
                        "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
                    )
                    .unwrap()
                ),
            }
            .to_vec()
        );

        assert_eq!(
            vec!["expiration", "1600000000"],
            TagStandard::Expiration(Timestamp::from(1600000000)).to_vec()
        );

        assert_eq!(
            vec!["content-warning", "reason"],
            TagStandard::ContentWarning {
                reason: Some(String::from("reason"))
            }
            .to_vec()
        );

        assert_eq!(
            vec!["subject", "textnote with subject"],
            TagStandard::Subject(String::from("textnote with subject")).to_vec()
        );

        assert_eq!(
            vec!["d", "test"],
            TagStandard::Identifier(String::from("test")).to_vec()
        );

        assert_eq!(
            vec![
                "p",
                "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d",
                "wss://relay.damus.io"
            ],
            TagStandard::PublicKey {
                public_key: PublicKey::from_str(
                    "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
                )
                .unwrap(),
                relay_url: Some(RelayUrl::parse("wss://relay.damus.io").unwrap()),
                alias: None,
                uppercase: false,
            }
            .to_vec()
        );

        assert_eq!(
            vec![
                "e",
                "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7",
            ],
            TagStandard::Event {
                event_id: EventId::from_hex(
                    "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7"
                )
                .unwrap(),
                relay_url: None,
                marker: None,
                public_key: None,
                uppercase: false,
            }
            .to_vec()
        );

        assert_eq!(
            vec![
                "e",
                "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7",
                "wss://relay.damus.io"
            ],
            TagStandard::Event {
                event_id: EventId::from_hex(
                    "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7"
                )
                .unwrap(),
                relay_url: Some(RelayUrl::parse("wss://relay.damus.io").unwrap()),
                marker: None,
                public_key: None,
                uppercase: false,
            }
            .to_vec()
        );

        assert_eq!(
            vec![
                "p",
                "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d",
                "spam"
            ],
            TagStandard::PublicKeyReport(
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
            TagStandard::EventReport(
                EventId::from_hex(
                    "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7"
                )
                .unwrap(),
                Report::Nudity,
            )
            .to_vec()
        );

        assert_eq!(
            vec!["nonce", "1", "20"],
            TagStandard::POW {
                nonce: 1,
                difficulty: 20
            }
            .to_vec()
        );

        assert_eq!(
            vec!["client", "voyage"],
            TagStandard::Client {
                name: String::from("voyage"),
                address: None
            }
            .to_vec()
        );

        assert_eq!(
            vec!["dep", "nostr"],
            TagStandard::Dependency(String::from("nostr")).to_vec()
        );

        assert_eq!(
            vec!["extension", "rs"],
            TagStandard::Extension(String::from("rs")).to_vec()
        );

        assert_eq!(
            vec!["license", "MIT"],
            TagStandard::License(String::from("MIT")).to_vec()
        );

        assert_eq!(
            vec!["runtime", "rustc 1.70.0"],
            TagStandard::Runtime(String::from("rustc 1.70.0")).to_vec()
        );

        assert_eq!(
            vec!["repo", "https://github.com/rust-nostr/nostr"],
            TagStandard::Repository(String::from("https://github.com/rust-nostr/nostr")).to_vec()
        );

        assert_eq!(
            vec!["client", "voyage", "30023:a695f6b60119d9521934a691347d9f78e8770b56da16bb255ee286ddf9fda919:ipsum"],
            TagStandard::Client {
                name: String::from("voyage"),
                address: Some((Coordinate::parse("30023:a695f6b60119d9521934a691347d9f78e8770b56da16bb255ee286ddf9fda919:ipsum").unwrap(), None))
            }.to_vec()
        );

        assert_eq!(
            vec!["client", "voyage", "30023:a695f6b60119d9521934a691347d9f78e8770b56da16bb255ee286ddf9fda919:ipsum"],
            TagStandard::Client {
                name: String::from("voyage"),
                address: Some((Coordinate::parse("30023:a695f6b60119d9521934a691347d9f78e8770b56da16bb255ee286ddf9fda919:ipsum").unwrap(), None))
            }.to_vec()
        );

        assert_eq!(
            vec!["client", "voyage", "30023:a695f6b60119d9521934a691347d9f78e8770b56da16bb255ee286ddf9fda919:ipsum", "wss://relay.damus.io"],
            TagStandard::Client {
                name: String::from("voyage"),
                address: Some((Coordinate::parse("30023:a695f6b60119d9521934a691347d9f78e8770b56da16bb255ee286ddf9fda919:ipsum").unwrap(), Some(RelayUrl::parse("wss://relay.damus.io").unwrap())))
            }.to_vec()
        );

        assert_eq!(
            vec![
                "a",
                "30023:a695f6b60119d9521934a691347d9f78e8770b56da16bb255ee286ddf9fda919:ipsum"
            ],
            TagStandard::Coordinate {
                coordinate: Coordinate::new(
                    Kind::LongFormTextNote,
                    PublicKey::from_str(
                        "a695f6b60119d9521934a691347d9f78e8770b56da16bb255ee286ddf9fda919"
                    )
                    .unwrap()
                )
                .identifier("ipsum"),
                relay_url: None,
                uppercase: false,
            }
            .to_vec()
        );

        assert_eq!(
            vec![
                "a",
                "30023:a695f6b60119d9521934a691347d9f78e8770b56da16bb255ee286ddf9fda919:ipsum",
                "wss://relay.nostr.org"
            ],
            TagStandard::Coordinate {
                coordinate: Coordinate::new(
                    Kind::LongFormTextNote,
                    PublicKey::from_str(
                        "a695f6b60119d9521934a691347d9f78e8770b56da16bb255ee286ddf9fda919"
                    )
                    .unwrap()
                )
                .identifier("ipsum"),
                relay_url: Some(RelayUrl::parse("wss://relay.nostr.org").unwrap()),
                uppercase: false,
            }
            .to_vec()
        );

        assert_eq!(
            vec![
                "p",
                "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d",
                "wss://relay.damus.io",
                "Speaker",
            ],
            TagStandard::PublicKeyLiveEvent {
                public_key: PublicKey::from_str(
                    "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
                )
                .unwrap(),
                relay_url: Some(RelayUrl::parse("wss://relay.damus.io").unwrap()),
                marker: LiveEventMarker::Speaker,
                proof: None
            }
            .to_vec()
        );

        assert_eq!(
            vec![
                "p",
                "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d",
                "",
                "Participant",
            ],
            TagStandard::PublicKeyLiveEvent {
                public_key: PublicKey::from_str(
                    "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
                )
                .unwrap(),
                relay_url: None,
                marker: LiveEventMarker::Participant,
                proof: None
            }
            .to_vec()
        );

        assert_eq!(
            vec![
                "p",
                "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d",
                "wss://relay.damus.io",
                "alias",
            ],
            TagStandard::PublicKey {
                public_key: PublicKey::from_str(
                    "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
                )
                .unwrap(),
                relay_url: Some(RelayUrl::parse("wss://relay.damus.io").unwrap()),
                alias: Some(String::from("alias")),
                uppercase: false,
            }
            .to_vec()
        );

        assert_eq!(
            vec![
                "e",
                "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7",
                "",
                "reply"
            ],
            TagStandard::Event {
                event_id: EventId::from_hex(
                    "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7"
                )
                .unwrap(),
                relay_url: None,
                marker: Some(Marker::Reply),
                public_key: None,
                uppercase: false,
            }
            .to_vec()
        );

        assert_eq!(
            vec![
                "e",
                "0000000000000000000000000000000000000000000000000000000000000001",
                "",
                "root",
                "0000000000000000000000000000000000000000000000000000000000000001",
            ],
            TagStandard::Event {
                event_id: EventId::from_hex(
                    "0000000000000000000000000000000000000000000000000000000000000001"
                )
                .unwrap(),
                relay_url: None,
                marker: Some(Marker::Root),
                public_key: Some(
                    PublicKey::parse(
                        "0000000000000000000000000000000000000000000000000000000000000001"
                    )
                    .unwrap()
                ),
                uppercase: false,
            }
            .to_vec()
        );

        assert_eq!(
            vec![
                "e",
                "0000000000000000000000000000000000000000000000000000000000000001",
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
                uppercase: false,
            }
            .to_vec()
        );

        assert_eq!(
            vec!["relay", "wss://relay.damus.io"],
            TagStandard::Relay(RelayUrl::parse("wss://relay.damus.io").unwrap()).to_vec()
        );

        assert_eq!(vec!["relay", "ALL_RELAYS"], TagStandard::AllRelays.to_vec());

        assert_eq!(
            vec![
                "delegation",
                "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d",
                "kind=1",
                "fd0954de564cae9923c2d8ee9ab2bf35bc19757f8e328a978958a2fcc950eaba0754148a203adec29b7b64080d0cf5a32bebedd768ea6eb421a6b751bb4584a8",
            ],
            TagStandard::Delegation {
                delegator: PublicKey::from_str(
                "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
            ).unwrap(), conditions: Conditions::from_str("kind=1").unwrap(), sig: Signature::from_str("fd0954de564cae9923c2d8ee9ab2bf35bc19757f8e328a978958a2fcc950eaba0754148a203adec29b7b64080d0cf5a32bebedd768ea6eb421a6b751bb4584a8").unwrap() }.to_vec()
        );

        assert_eq!(
            vec!["lnurl", "lnurl1dp68gurn8ghj7um5v93kketj9ehx2amn9uh8wetvdskkkmn0wahz7mrww4excup0dajx2mrv92x9xp"],
            TagStandard::Lnurl(String::from("lnurl1dp68gurn8ghj7um5v93kketj9ehx2amn9uh8wetvdskkkmn0wahz7mrww4excup0dajx2mrv92x9xp")).to_vec(),
        );

        assert_eq!(
            vec![
                "p",
                "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d",
                "wss://relay.damus.io",
                "Host",
                "a5d9290ef9659083c490b303eb7ee41356d8778ff19f2f91776c8dc4443388a64ffcf336e61af4c25c05ac3ae952d1ced889ed655b67790891222aaa15b99fdd"
            ],
            TagStandard::PublicKeyLiveEvent {
                public_key: PublicKey::from_hex(
                    "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
                ).unwrap(),
                relay_url: Some(RelayUrl::parse("wss://relay.damus.io").unwrap()),
                marker: LiveEventMarker::Host,
                proof: Some(Signature::from_str("a5d9290ef9659083c490b303eb7ee41356d8778ff19f2f91776c8dc4443388a64ffcf336e61af4c25c05ac3ae952d1ced889ed655b67790891222aaa15b99fdd").unwrap())
            }.to_vec()
        );

        assert_eq!(
            vec!["L", "#t"],
            TagStandard::LabelNamespace("#t".to_string()).to_vec()
        );

        assert_eq!(
            vec!["l", "IT-MI"],
            TagStandard::Label {
                value: "IT-MI".to_string(),
                namespace: None
            }
            .to_vec()
        );

        assert_eq!(
            vec!["l", "IT-MI", "ISO-3166-2"],
            TagStandard::Label {
                value: "IT-MI".to_string(),
                namespace: Some("ISO-3166-2".to_string())
            }
            .to_vec()
        );

        assert_eq!(
            vec!["r", "wss://atlas.nostr.land/"],
            TagStandard::RelayMetadata {
                relay_url: RelayUrl::parse("wss://atlas.nostr.land/").unwrap(),
                metadata: None
            }
            .to_vec()
        );

        assert_eq!(
            vec!["r", "wss://atlas.nostr.land/", "read"],
            TagStandard::RelayMetadata {
                relay_url: RelayUrl::parse("wss://atlas.nostr.land/").unwrap(),
                metadata: Some(RelayMetadata::Read)
            }
            .to_vec()
        );

        assert_eq!(
            vec!["r", "wss://atlas.nostr.land", "write"],
            TagStandard::RelayMetadata {
                relay_url: RelayUrl::parse("wss://atlas.nostr.land").unwrap(),
                metadata: Some(RelayMetadata::Write)
            }
            .to_vec()
        );

        assert_eq!(
            vec!["r", "5e664e5a7845cd1373c79f580ca4fe29ab5b34d2", "euc"],
            TagStandard::GitEarliestUniqueCommitId(
                Sha1Hash::from_str("5e664e5a7845cd1373c79f580ca4fe29ab5b34d2").unwrap()
            )
            .to_vec()
        );

        assert_eq!(
            vec!["clone", "https://github.com/rust-nostr/nostr.git",],
            TagStandard::GitClone(vec![
                Url::parse("https://github.com/rust-nostr/nostr.git").unwrap()
            ])
            .to_vec()
        );

        assert_eq!(
            vec![
                "maintainers",
                "32e1827635450ebb3c5a7d12c1f8e7b2b514439ac10a67eef3d9fd9c5c68e245",
                "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
            ],
            TagStandard::GitMaintainers(vec![
                PublicKey::from_hex(
                    "32e1827635450ebb3c5a7d12c1f8e7b2b514439ac10a67eef3d9fd9c5c68e245"
                )
                .unwrap(),
                PublicKey::from_hex(
                    "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
                )
                .unwrap(),
            ])
            .to_vec()
        );

        assert_eq!(
            vec![
                "web",
                "https://rust-nostr.org/",
                "https://github.com/rust-nostr",
            ],
            TagStandard::Web(vec![
                Url::parse("https://rust-nostr.org").unwrap(),
                Url::parse("https://github.com/rust-nostr").unwrap(),
            ])
            .to_vec()
        );
    }

    #[test]
    fn test_tag_standard_parsing() {
        assert_eq!(TagStandard::parse(&["-"]).unwrap(), TagStandard::Protected);

        assert_eq!(
            TagStandard::parse(&["alt", "something"]).unwrap(),
            TagStandard::Alt(String::from("something"))
        );

        assert_eq!(
            TagStandard::parse(&["content-warning"]).unwrap(),
            TagStandard::ContentWarning { reason: None }
        );

        assert_eq!(
            TagStandard::parse(&[
                "p",
                "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
            ])
            .unwrap(),
            TagStandard::public_key(
                PublicKey::from_str(
                    "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
                )
                .unwrap()
            )
        );

        assert_eq!(
            TagStandard::parse(&[
                "e",
                "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7"
            ])
            .unwrap(),
            TagStandard::event(
                EventId::from_hex(
                    "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7"
                )
                .unwrap()
            )
        );

        assert_eq!(
            TagStandard::parse(&[
                "E",
                "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7"
            ]),
            Ok(TagStandard::Event {
                event_id: EventId::from_hex(
                    "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7"
                )
                .unwrap(),
                relay_url: None,
                marker: None,
                public_key: None,
                uppercase: true
            })
        );

        assert_eq!(
            TagStandard::parse(&[
                "q",
                "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7"
            ])
            .unwrap(),
            TagStandard::Quote {
                event_id: EventId::from_hex(
                    "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7"
                )
                .unwrap(),
                relay_url: None,
                public_key: None,
            }
        );

        assert_eq!(
            TagStandard::parse(&[
                "q",
                "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7",
                "wss://relay.damus.io"
            ])
            .unwrap(),
            TagStandard::Quote {
                event_id: EventId::from_hex(
                    "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7"
                )
                .unwrap(),
                relay_url: Some(RelayUrl::parse("wss://relay.damus.io").unwrap()),
                public_key: None,
            }
        );

        assert_eq!(
            TagStandard::parse(&[
                "q",
                "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7",
                "",
                "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
            ])
            .unwrap(),
            TagStandard::Quote {
                event_id: EventId::from_hex(
                    "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7"
                )
                .unwrap(),
                relay_url: None,
                public_key: Some(
                    PublicKey::from_hex(
                        "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
                    )
                    .unwrap()
                ),
            }
        );

        assert_eq!(
            TagStandard::parse(&["expiration", "1600000000"]).unwrap(),
            TagStandard::Expiration(Timestamp::from(1600000000))
        );

        assert_eq!(
            TagStandard::parse(&["content-warning", "reason"]).unwrap(),
            TagStandard::ContentWarning {
                reason: Some(String::from("reason"))
            }
        );

        assert_eq!(
            TagStandard::parse(&["subject", "textnote with subject"]).unwrap(),
            TagStandard::Subject(String::from("textnote with subject"))
        );

        assert_eq!(
            TagStandard::parse(&["d", "test"]).unwrap(),
            TagStandard::Identifier(String::from("test"))
        );

        assert_eq!(
            TagStandard::parse(&["r", "https://example.com"]).unwrap(),
            TagStandard::Reference(String::from("https://example.com"))
        );

        assert_eq!(
            TagStandard::parse(&["i", "isbn:9780765382030"]).unwrap(),
            TagStandard::ExternalContent {
                content: ExternalContentId::Book(String::from("9780765382030")),
                hint: None,
                uppercase: false,
            }
        );

        assert_eq!(
            TagStandard::parse(&[
                "i",
                "podcast:guid:c90e609a-df1e-596a-bd5e-57bcc8aad6cc",
                "https://podcastindex.org/podcast/c90e609a-df1e-596a-bd5e-57bcc8aad6cc"
            ])
            .unwrap(),
            TagStandard::ExternalContent {
                content: ExternalContentId::PodcastFeed(String::from(
                    "c90e609a-df1e-596a-bd5e-57bcc8aad6cc"
                )),
                hint: Some(
                    Url::parse(
                        "https://podcastindex.org/podcast/c90e609a-df1e-596a-bd5e-57bcc8aad6cc"
                    )
                    .unwrap()
                ),
                uppercase: false,
            }
        );

        assert_eq!(
            TagStandard::parse(&["i", "github:12345678", "abcdefghijklmnop"]).unwrap(),
            TagStandard::ExternalIdentity(Identity {
                platform: ExternalIdentity::GitHub,
                ident: "12345678".to_string(),
                proof: "abcdefghijklmnop".to_string()
            })
        );

        assert_eq!(
            TagStandard::parse(&[
                "p",
                "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d",
                "wss://relay.damus.io"
            ])
            .unwrap(),
            TagStandard::PublicKey {
                public_key: PublicKey::from_str(
                    "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
                )
                .unwrap(),
                relay_url: Some(RelayUrl::parse("wss://relay.damus.io").unwrap()),
                alias: None,
                uppercase: false
            }
        );

        assert_eq!(
            TagStandard::parse(&[
                "e",
                "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7",
                ""
            ])
            .unwrap(),
            TagStandard::Event {
                event_id: EventId::from_hex(
                    "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7"
                )
                .unwrap(),
                relay_url: None,
                marker: None,
                public_key: None,
                uppercase: false,
            }
        );

        assert_eq!(
            TagStandard::parse(&[
                "e",
                "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7",
                "wss://relay.damus.io"
            ])
            .unwrap(),
            TagStandard::Event {
                event_id: EventId::from_hex(
                    "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7"
                )
                .unwrap(),
                relay_url: Some(RelayUrl::parse("wss://relay.damus.io").unwrap()),
                marker: None,
                public_key: None,
                uppercase: false,
            }
        );

        assert_eq!(
            TagStandard::parse(&[
                "p",
                "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d",
                "impersonation"
            ])
            .unwrap(),
            TagStandard::PublicKeyReport(
                PublicKey::from_str(
                    "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
                )
                .unwrap(),
                Report::Impersonation
            )
        );

        assert_eq!(
            TagStandard::parse(&[
                "p",
                "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d",
                "other"
            ])
            .unwrap(),
            TagStandard::PublicKeyReport(
                PublicKey::from_str(
                    "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
                )
                .unwrap(),
                Report::Other
            )
        );

        assert_eq!(
            TagStandard::parse(&[
                "e",
                "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7",
                "profanity"
            ])
            .unwrap(),
            TagStandard::EventReport(
                EventId::from_hex(
                    "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7"
                )
                .unwrap(),
                Report::Profanity
            )
        );

        assert_eq!(
            TagStandard::parse(&[
                "e",
                "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7",
                "malware"
            ])
            .unwrap(),
            TagStandard::EventReport(
                EventId::from_hex(
                    "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7"
                )
                .unwrap(),
                Report::Malware
            )
        );

        assert_eq!(
            TagStandard::parse(&["nonce", "1", "20"]).unwrap(),
            TagStandard::POW {
                nonce: 1,
                difficulty: 20
            }
        );

        assert_eq!(
            TagStandard::parse(&["client", "voyage"]).unwrap(),
            TagStandard::Client {
                name: String::from("voyage"),
                address: None
            }
        );

        assert_eq!(
            TagStandard::parse(&["dep", "nostr"]).unwrap(),
            TagStandard::Dependency(String::from("nostr"))
        );

        assert_eq!(
            TagStandard::parse(&["extension", "rs"]).unwrap(),
            TagStandard::Extension(String::from("rs"))
        );

        assert_eq!(
            TagStandard::parse(&["license", "MIT"]).unwrap(),
            TagStandard::License(String::from("MIT"))
        );

        assert_eq!(
            TagStandard::parse(&["runtime", "rustc 1.70.0"]).unwrap(),
            TagStandard::Runtime(String::from("rustc 1.70.0"))
        );

        assert_eq!(
            TagStandard::parse(&["repo", "https://github.com/rust-nostr/nostr"]).unwrap(),
            TagStandard::Repository(String::from("https://github.com/rust-nostr/nostr"))
        );

        assert_eq!(
            TagStandard::parse(&["client", "voyage", "30023:a695f6b60119d9521934a691347d9f78e8770b56da16bb255ee286ddf9fda919:ipsum"]).unwrap(),
            TagStandard::Client {
                name: String::from("voyage"),
                address: Some((Coordinate::parse("30023:a695f6b60119d9521934a691347d9f78e8770b56da16bb255ee286ddf9fda919:ipsum").unwrap(), None))
            }
        );

        assert_eq!(
            TagStandard::parse(&["client", "voyage", "30023:a695f6b60119d9521934a691347d9f78e8770b56da16bb255ee286ddf9fda919:ipsum", ""]).unwrap(),
            TagStandard::Client {
                name: String::from("voyage"),
                address: Some((Coordinate::parse("30023:a695f6b60119d9521934a691347d9f78e8770b56da16bb255ee286ddf9fda919:ipsum").unwrap(), None))
            }
        );

        assert_eq!(
            TagStandard::parse(&["client", "voyage", "30023:a695f6b60119d9521934a691347d9f78e8770b56da16bb255ee286ddf9fda919:ipsum", "wss://relay.damus.io"]).unwrap(),
            TagStandard::Client {
                name: String::from("voyage"),
                address: Some((Coordinate::parse("30023:a695f6b60119d9521934a691347d9f78e8770b56da16bb255ee286ddf9fda919:ipsum").unwrap(), Some(RelayUrl::parse("wss://relay.damus.io").unwrap())))
            }
        );

        assert_eq!(
            TagStandard::parse(&[
                "a",
                "30023:a695f6b60119d9521934a691347d9f78e8770b56da16bb255ee286ddf9fda919:ipsum",
                "wss://relay.nostr.org"
            ])
            .unwrap(),
            TagStandard::Coordinate {
                coordinate: Coordinate::new(
                    Kind::LongFormTextNote,
                    PublicKey::from_str(
                        "a695f6b60119d9521934a691347d9f78e8770b56da16bb255ee286ddf9fda919"
                    )
                    .unwrap()
                )
                .identifier("ipsum"),
                relay_url: Some(RelayUrl::parse("wss://relay.nostr.org").unwrap()),
                uppercase: false,
            }
        );

        assert_eq!(
            TagStandard::parse(&[
                "A",
                "30023:a695f6b60119d9521934a691347d9f78e8770b56da16bb255ee286ddf9fda919:ipsum",
                "wss://relay.nostr.org"
            ])
            .unwrap(),
            TagStandard::Coordinate {
                coordinate: Coordinate::new(
                    Kind::LongFormTextNote,
                    PublicKey::from_str(
                        "a695f6b60119d9521934a691347d9f78e8770b56da16bb255ee286ddf9fda919"
                    )
                    .unwrap()
                )
                .identifier("ipsum"),
                relay_url: Some(RelayUrl::parse("wss://relay.nostr.org").unwrap()),
                uppercase: true,
            }
        );

        assert_eq!(
            TagStandard::parse(&["r", "wss://atlas.nostr.land/"]).unwrap(),
            TagStandard::RelayMetadata {
                relay_url: RelayUrl::parse("wss://atlas.nostr.land/").unwrap(),
                metadata: None
            }
        );

        assert_eq!(
            TagStandard::parse(&["r", "wss://atlas.nostr.land", "read"]).unwrap(),
            TagStandard::RelayMetadata {
                relay_url: RelayUrl::parse("wss://atlas.nostr.land").unwrap(),
                metadata: Some(RelayMetadata::Read)
            }
        );

        assert_eq!(
            TagStandard::parse(&["r", "wss://atlas.nostr.land", "write"]).unwrap(),
            TagStandard::RelayMetadata {
                relay_url: RelayUrl::parse("wss://atlas.nostr.land").unwrap(),
                metadata: Some(RelayMetadata::Write)
            }
        );

        assert_eq!(
            TagStandard::parse(&[
                "p",
                "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d",
                "wss://relay.damus.io/",
                "alias",
            ])
            .unwrap(),
            TagStandard::PublicKey {
                public_key: PublicKey::from_str(
                    "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
                )
                .unwrap(),
                relay_url: Some(RelayUrl::parse("wss://relay.damus.io/").unwrap()),
                alias: Some(String::from("alias")),
                uppercase: false,
            }
        );

        assert_eq!(
            TagStandard::parse(&[
                "e",
                "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7",
                "",
                "reply"
            ])
            .unwrap(),
            TagStandard::Event {
                event_id: EventId::from_hex(
                    "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7"
                )
                .unwrap(),
                relay_url: None,
                marker: Some(Marker::Reply),
                public_key: None,
                uppercase: false,
            }
        );

        assert_eq!(
            TagStandard::parse(&[
                "e",
                "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7",
                "",
                "reply",
                "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
            ])
            .unwrap(),
            TagStandard::Event {
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
                uppercase: false,
            }
        );

        assert_eq!(
            TagStandard::parse(&[
                "e",
                "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7",
                "",
                "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
            ])
            .unwrap(),
            TagStandard::Event {
                event_id: EventId::from_hex(
                    "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7"
                )
                .unwrap(),
                relay_url: None,
                marker: None,
                public_key: Some(
                    PublicKey::from_hex(
                        "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
                    )
                    .unwrap()
                ),
                uppercase: false,
            }
        );

        assert_eq!(
            TagStandard::parse(&[
                "delegation",
                "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d",
                "kind=1",
                "fd0954de564cae9923c2d8ee9ab2bf35bc19757f8e328a978958a2fcc950eaba0754148a203adec29b7b64080d0cf5a32bebedd768ea6eb421a6b751bb4584a8",
            ]).unwrap(),
            TagStandard::Delegation { delegator: PublicKey::from_str(
                "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
            ).unwrap(), conditions: Conditions::from_str("kind=1").unwrap(), sig: Signature::from_str("fd0954de564cae9923c2d8ee9ab2bf35bc19757f8e328a978958a2fcc950eaba0754148a203adec29b7b64080d0cf5a32bebedd768ea6eb421a6b751bb4584a8").unwrap() }
        );

        assert_eq!(
            TagStandard::parse(&["relay", "wss://relay.damus.io"]).unwrap(),
            TagStandard::Relay(RelayUrl::parse("wss://relay.damus.io").unwrap())
        );

        assert_eq!(
            TagStandard::parse(&["relay", "ALL_RELAYS"]).unwrap(),
            TagStandard::AllRelays
        );

        assert_eq!(
            TagStandard::parse(&[
                "relays",
                "wss://relay.damus.io/",
                "wss://nostr-relay.wlvs.space/",
                "wss://nostr.fmt.wiz.biz/"
            ])
            .unwrap(),
            TagStandard::Relays(vec![
                RelayUrl::parse("wss://relay.damus.io/").unwrap(),
                RelayUrl::parse("wss://nostr-relay.wlvs.space/").unwrap(),
                RelayUrl::parse("wss://nostr.fmt.wiz.biz").unwrap(),
            ])
        );

        assert_eq!(
            TagStandard::parse(&[
                "bolt11",
                "lnbc10u1p3unwfusp5t9r3yymhpfqculx78u027lxspgxcr2n2987mx2j55nnfs95nxnzqpp5jmrh92pfld78spqs78v9euf2385t83uvpwk9ldrlvf6ch7tpascqhp5zvkrmemgth3tufcvflmzjzfvjt023nazlhljz2n9hattj4f8jq8qxqyjw5qcqpjrzjqtc4fc44feggv7065fqe5m4ytjarg3repr5j9el35xhmtfexc42yczarjuqqfzqqqqqqqqlgqqqqqqgq9q9qxpqysgq079nkq507a5tw7xgttmj4u990j7wfggtrasah5gd4ywfr2pjcn29383tphp4t48gquelz9z78p4cq7ml3nrrphw5w6eckhjwmhezhnqpy6gyf0"]).unwrap(),
            TagStandard::Bolt11("lnbc10u1p3unwfusp5t9r3yymhpfqculx78u027lxspgxcr2n2987mx2j55nnfs95nxnzqpp5jmrh92pfld78spqs78v9euf2385t83uvpwk9ldrlvf6ch7tpascqhp5zvkrmemgth3tufcvflmzjzfvjt023nazlhljz2n9hattj4f8jq8qxqyjw5qcqpjrzjqtc4fc44feggv7065fqe5m4ytjarg3repr5j9el35xhmtfexc42yczarjuqqfzqqqqqqqqlgqqqqqqgq9q9qxpqysgq079nkq507a5tw7xgttmj4u990j7wfggtrasah5gd4ywfr2pjcn29383tphp4t48gquelz9z78p4cq7ml3nrrphw5w6eckhjwmhezhnqpy6gyf0".to_string())
        );

        assert_eq!(
            TagStandard::parse(&[
                "preimage",
                "5d006d2cf1e73c7148e7519a4c68adc81642ce0e25a432b2434c99f97344c15f"
            ])
            .unwrap(),
            TagStandard::Preimage(
                "5d006d2cf1e73c7148e7519a4c68adc81642ce0e25a432b2434c99f97344c15f".to_string()
            )
        );

        assert_eq!(
            TagStandard::parse(&[
                "description",
                "{\"pubkey\":\"32e1827635450ebb3c5a7d12c1f8e7b2b514439ac10a67eef3d9fd9c5c68e245\",\"content\":\"\",\"id\":\"d9cc14d50fcb8c27539aacf776882942c1a11ea4472f8cdec1dea82fab66279d\",\"created_at\":1674164539,\"sig\":\"77127f636577e9029276be060332ea565deaf89ff215a494ccff16ae3f757065e2bc59b2e8c113dd407917a010b3abd36c8d7ad84c0e3ab7dab3a0b0caa9835d\",\"kind\":9734,\"tags\":[[\"e\",\"3624762a1274dd9636e0c552b53086d70bc88c165bc4dc0f9e836a1eaf86c3b8\"],[\"p\",\"32e1827635450ebb3c5a7d12c1f8e7b2b514439ac10a67eef3d9fd9c5c68e245\"],[\"relays\",\"wss://relay.damus.io\",\"wss://nostr-relay.wlvs.space\",\"wss://nostr.fmt.wiz.biz\",\"wss://relay.nostr.bg\",\"wss://nostr.oxtr.dev\",\"wss://nostr.v0l.io\",\"wss://brb.io\",\"wss://nostr.bitcoiner.social\",\"ws://monad.jb55.com:8080\",\"wss://relay.snort.social\"]]}"
            ]).unwrap(),
            TagStandard::Description("{\"pubkey\":\"32e1827635450ebb3c5a7d12c1f8e7b2b514439ac10a67eef3d9fd9c5c68e245\",\"content\":\"\",\"id\":\"d9cc14d50fcb8c27539aacf776882942c1a11ea4472f8cdec1dea82fab66279d\",\"created_at\":1674164539,\"sig\":\"77127f636577e9029276be060332ea565deaf89ff215a494ccff16ae3f757065e2bc59b2e8c113dd407917a010b3abd36c8d7ad84c0e3ab7dab3a0b0caa9835d\",\"kind\":9734,\"tags\":[[\"e\",\"3624762a1274dd9636e0c552b53086d70bc88c165bc4dc0f9e836a1eaf86c3b8\"],[\"p\",\"32e1827635450ebb3c5a7d12c1f8e7b2b514439ac10a67eef3d9fd9c5c68e245\"],[\"relays\",\"wss://relay.damus.io\",\"wss://nostr-relay.wlvs.space\",\"wss://nostr.fmt.wiz.biz\",\"wss://relay.nostr.bg\",\"wss://nostr.oxtr.dev\",\"wss://nostr.v0l.io\",\"wss://brb.io\",\"wss://nostr.bitcoiner.social\",\"ws://monad.jb55.com:8080\",\"wss://relay.snort.social\"]]}".to_string())
        );

        assert_eq!(
            TagStandard::parse(&["amount", "10000"]).unwrap(),
            TagStandard::Amount {
                millisats: 10_000,
                bolt11: None
            }
        );

        assert_eq!(
            TagStandard::parse(&["L", "#t"]).unwrap(),
            TagStandard::LabelNamespace("#t".to_string())
        );

        assert_eq!(
            TagStandard::parse(&["l", "IT-MI"]).unwrap(),
            TagStandard::Label {
                value: "IT-MI".to_string(),
                namespace: None
            }
        );

        assert_eq!(
            TagStandard::parse(&["l", "IT-MI", "ISO-3166-2"]).unwrap(),
            TagStandard::Label {
                value: "IT-MI".to_string(),
                namespace: Some("ISO-3166-2".to_string())
            }
        );

        assert_eq!(
            TagStandard::parse(&["r", "5e664e5a7845cd1373c79f580ca4fe29ab5b34d2", "euc"]).unwrap(),
            TagStandard::GitEarliestUniqueCommitId(
                Sha1Hash::from_str("5e664e5a7845cd1373c79f580ca4fe29ab5b34d2").unwrap()
            )
        );

        assert_eq!(
            TagStandard::parse(&["clone", "https://github.com/rust-nostr/nostr.git"]).unwrap(),
            TagStandard::GitClone(vec![
                Url::parse("https://github.com/rust-nostr/nostr.git").unwrap()
            ])
        );

        assert_eq!(
            TagStandard::parse(&[
                "maintainers",
                "32e1827635450ebb3c5a7d12c1f8e7b2b514439ac10a67eef3d9fd9c5c68e245",
                "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
            ])
            .unwrap(),
            TagStandard::GitMaintainers(vec![
                PublicKey::from_hex(
                    "32e1827635450ebb3c5a7d12c1f8e7b2b514439ac10a67eef3d9fd9c5c68e245"
                )
                .unwrap(),
                PublicKey::from_hex(
                    "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
                )
                .unwrap(),
            ])
        );

        assert_eq!(
            TagStandard::parse(&[
                "web",
                "https://rust-nostr.org/",
                "https://github.com/rust-nostr",
            ])
            .unwrap(),
            TagStandard::Web(vec![
                Url::parse("https://rust-nostr.org").unwrap(),
                Url::parse("https://github.com/rust-nostr").unwrap(),
            ])
        );

        assert_eq!(
            TagStandard::parse(&["t", "Nostr"]),
            Err(Error::UnknownStandardizedTag)
        );
        assert!(TagStandard::parse(&["t", "nostr"]).is_ok());
        assert!(TagStandard::parse(&["t", "سلام"]).is_ok());
    }
}
