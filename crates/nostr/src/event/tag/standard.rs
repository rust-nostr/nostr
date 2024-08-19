// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Standardized tags

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::str::FromStr;

use bitcoin::hashes::sha256::Hash as Sha256Hash;
use bitcoin::secp256k1::schnorr::Signature;

use super::{Error, TagKind};
use crate::event::id::EventId;
use crate::nips::nip01::Coordinate;
use crate::nips::nip10::Marker;
use crate::nips::nip26::Conditions;
use crate::nips::nip39::Identity;
use crate::nips::nip48::Protocol;
use crate::nips::nip53::{LiveEventMarker, LiveEventStatus};
use crate::nips::nip56::Report;
use crate::nips::nip65::RelayMetadata;
use crate::nips::nip90::DataVendingMachineStatus;
use crate::nips::nip98::HttpMethod;
use crate::types::url::Url;
use crate::{
    Alphabet, Event, ImageDimensions, JsonUtil, Kind, PublicKey, SingleLetterTag, Timestamp,
    UncheckedUrl,
};

/// Standardized tag
#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TagStandard {
    /// Event
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md> and <https://github.com/nostr-protocol/nips/blob/master/10.md>
    Event {
        event_id: EventId,
        relay_url: Option<UncheckedUrl>,
        marker: Option<Marker>,
        /// Should be the public key of the author of the referenced event
        public_key: Option<PublicKey>,
    },
    /// Report event
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/56.md>
    EventReport(EventId, Report),
    /// Public Key
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    PublicKey {
        public_key: PublicKey,
        relay_url: Option<UncheckedUrl>,
        alias: Option<String>,
        /// Whether the p tag is an uppercase P or not
        uppercase: bool,
    },
    /// Report public key
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/56.md>
    PublicKeyReport(PublicKey, Report),
    PublicKeyLiveEvent {
        public_key: PublicKey,
        relay_url: Option<UncheckedUrl>,
        marker: LiveEventMarker,
        proof: Option<Signature>,
    },
    Reference(String),
    /// Relay Metadata
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/65.md>
    RelayMetadata {
        relay_url: Url,
        metadata: Option<RelayMetadata>,
    },
    Hashtag(String),
    Geohash(String),
    Identifier(String),
    ExternalIdentity(Identity),
    Coordinate {
        coordinate: Coordinate,
        relay_url: Option<UncheckedUrl>,
    },
    Kind(Kind),
    Relay(UncheckedUrl),
    /// Proof of Work
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/13.md>
    POW {
        nonce: u128,
        difficulty: u8,
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
    Image(UncheckedUrl, Option<ImageDimensions>),
    Thumb(UncheckedUrl, Option<ImageDimensions>),
    Summary(String),
    Description(String),
    Bolt11(String),
    Preimage(String),
    Relays(Vec<UncheckedUrl>),
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
    Sha256(Sha256Hash),
    Size(usize),
    Dim(ImageDimensions),
    Magnet(String),
    Blurhash(String),
    Streaming(UncheckedUrl),
    Recording(UncheckedUrl),
    Starts(Timestamp),
    Ends(Timestamp),
    LiveEventStatus(LiveEventStatus),
    CurrentParticipants(u64),
    TotalParticipants(u64),
    AbsoluteURL(UncheckedUrl),
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
        /// Name given for the emoji, which MUST be comprised of only alphanumeric characters and underscores
        shortcode: String,
        /// URL to the corresponding image file of the emoji
        url: UncheckedUrl,
    },
    Encrypted,
    Request(Event),
    DataVendingMachineStatus {
        status: DataVendingMachineStatus,
        extra_info: Option<String>,
    },
    Word(String),
    /// Label namespace
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/32.md>
    LabelNamespace(String),
    /// Label
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/32.md>
    Label(Vec<String>),
    /// Protected event
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/70.md>
    Protected,
    /// A short human-readable plaintext summary of what that event is about
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/31.md>
    Alt(String),
}

impl TagStandard {
    /// Parse [`Tag`] from slice of string
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
                    uppercase: false,
                } => {
                    return parse_a_tag(tag);
                }
                // Parse `e` tag
                SingleLetterTag {
                    character: Alphabet::E,
                    uppercase: false,
                } => {
                    return parse_e_tag(tag);
                }
                // Parse `l` tag
                SingleLetterTag {
                    character: Alphabet::L,
                    uppercase: false,
                } => {
                    let labels = tag.iter().skip(1).map(|u| u.as_ref().to_string()).collect();
                    return Ok(Self::Label(labels));
                }
                // Parse `p` tag
                SingleLetterTag {
                    character: Alphabet::P,
                    uppercase,
                } => {
                    return parse_p_tag(tag, uppercase);
                }
                _ => (), // Covered later
            },
            TagKind::Anon => {
                return Ok(Self::Anon {
                    msg: extract_optional_string(tag, 1).map(|s| s.to_string()),
                })
            }
            TagKind::ContentWarning => {
                return Ok(Self::ContentWarning {
                    reason: extract_optional_string(tag, 1).map(|s| s.to_string()),
                })
            }
            TagKind::Delegation => return parse_delegation_tag(tag),
            TagKind::Encrypted => return Ok(Self::Encrypted),
            TagKind::Protected => return Ok(Self::Protected),
            TagKind::Relays => {
                // Relays vec is of unknown length so checked here based on kind
                let urls = tag
                    .iter()
                    .skip(1)
                    .map(|u| UncheckedUrl::from(u.as_ref()))
                    .collect::<Vec<UncheckedUrl>>();
                return Ok(Self::Relays(urls));
            }
            _ => (), // Covered later
        };

        let tag_len: usize = tag.len();

        if tag_len == 2 {
            let tag_1: &str = tag[1].as_ref();

            return match tag_kind {
                TagKind::SingleLetter(SingleLetterTag {
                    character: Alphabet::R,
                    uppercase: false,
                }) => {
                    if tag_1.starts_with("ws://") || tag_1.starts_with("wss://") {
                        Ok(Self::RelayMetadata {
                            relay_url: Url::parse(tag_1)?,
                            metadata: None,
                        })
                    } else {
                        Ok(Self::Reference(tag_1.to_string()))
                    }
                }
                TagKind::SingleLetter(SingleLetterTag {
                    character: Alphabet::T,
                    uppercase: false,
                }) => Ok(Self::Hashtag(tag_1.to_string())),
                TagKind::SingleLetter(SingleLetterTag {
                    character: Alphabet::G,
                    uppercase: false,
                }) => Ok(Self::Geohash(tag_1.to_string())),
                TagKind::SingleLetter(SingleLetterTag {
                    character: Alphabet::D,
                    uppercase: false,
                }) => Ok(Self::Identifier(tag_1.to_string())),
                TagKind::SingleLetter(SingleLetterTag {
                    character: Alphabet::K,
                    uppercase: false,
                }) => Ok(Self::Kind(Kind::from_str(tag_1)?)),
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
                }) => Ok(Self::AbsoluteURL(UncheckedUrl::from(tag_1))),
                TagKind::Relay => Ok(Self::Relay(UncheckedUrl::from(tag_1))),
                TagKind::Expiration => Ok(Self::Expiration(Timestamp::from_str(tag_1)?)),
                TagKind::Subject => Ok(Self::Subject(tag_1.to_string())),
                TagKind::Challenge => Ok(Self::Challenge(tag_1.to_string())),
                TagKind::Title => Ok(Self::Title(tag_1.to_string())),
                TagKind::Image => Ok(Self::Image(UncheckedUrl::from(tag_1), None)),
                TagKind::Thumb => Ok(Self::Thumb(UncheckedUrl::from(tag_1), None)),
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
                TagKind::Streaming => Ok(Self::Streaming(UncheckedUrl::from(tag_1))),
                TagKind::Recording => Ok(Self::Recording(UncheckedUrl::from(tag_1))),
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
                TagKind::Method => Ok(Self::Method(HttpMethod::from_str(tag_1)?)),
                TagKind::Payload => Ok(Self::Payload(Sha256Hash::from_str(tag_1)?)),
                TagKind::Request => Ok(Self::Request(Event::from_json(tag_1)?)),
                TagKind::Word => Ok(Self::Word(tag_1.to_string())),
                TagKind::SingleLetter(SingleLetterTag {
                    character: Alphabet::L,
                    uppercase: true,
                }) => Ok(Self::LabelNamespace(tag_1.to_string())),
                TagKind::Alt => Ok(Self::Alt(tag_1.to_string())),
                TagKind::Dim => Ok(Self::Dim(ImageDimensions::from_str(tag_1)?)),
                _ => Err(Error::UnknownStardardizedTag),
            };
        }

        if tag_len == 3 {
            let tag_1: &str = tag[1].as_ref();
            let tag_2: &str = tag[2].as_ref();

            return match tag_kind {
                TagKind::SingleLetter(SingleLetterTag {
                    character: Alphabet::I,
                    uppercase: false,
                }) => Ok(Self::ExternalIdentity(Identity::new(tag_1, tag_2)?)),
                TagKind::Nonce => Ok(Self::POW {
                    nonce: tag_1.parse()?,
                    difficulty: tag_2.parse()?,
                }),
                TagKind::Image => Ok(Self::Image(
                    UncheckedUrl::from(tag_1),
                    Some(ImageDimensions::from_str(tag_2)?),
                )),
                TagKind::Thumb => Ok(Self::Thumb(
                    UncheckedUrl::from(tag_1),
                    Some(ImageDimensions::from_str(tag_2)?),
                )),
                TagKind::Aes256Gcm => Ok(Self::Aes256Gcm {
                    key: tag_1.to_string(),
                    iv: tag_2.to_string(),
                }),
                TagKind::SingleLetter(SingleLetterTag {
                    character: Alphabet::R,
                    uppercase: false,
                }) => {
                    if (tag_1.starts_with("ws://") || tag_1.starts_with("wss://"))
                        && !tag_2.is_empty()
                    {
                        Ok(Self::RelayMetadata {
                            relay_url: Url::parse(tag_1)?,
                            metadata: Some(RelayMetadata::from_str(tag_2)?),
                        })
                    } else {
                        Err(Error::UnknownStardardizedTag)
                    }
                }
                TagKind::Proxy => Ok(Self::Proxy {
                    id: tag_1.to_string(),
                    protocol: Protocol::from(tag_2),
                }),
                TagKind::Emoji => Ok(Self::Emoji {
                    shortcode: tag_1.to_string(),
                    url: UncheckedUrl::from(tag_2),
                }),
                TagKind::Status => match DataVendingMachineStatus::from_str(tag_1) {
                    Ok(status) => Ok(Self::DataVendingMachineStatus {
                        status,
                        extra_info: Some(tag_2.to_string()),
                    }),
                    Err(_) => Err(Error::UnknownStardardizedTag),
                },
                _ => Err(Error::UnknownStardardizedTag),
            };
        }

        Err(Error::UnknownStardardizedTag)
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
            Self::Event { .. } | Self::EventReport(..) => TagKind::SingleLetter(SingleLetterTag {
                character: Alphabet::E,
                uppercase: false,
            }),
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
            Self::ExternalIdentity(..) => TagKind::SingleLetter(SingleLetterTag {
                character: Alphabet::I,
                uppercase: false,
            }),
            Self::Coordinate { .. } => TagKind::SingleLetter(SingleLetterTag {
                character: Alphabet::A,
                uppercase: false,
            }),
            Self::Kind(..) => TagKind::SingleLetter(SingleLetterTag {
                character: Alphabet::K,
                uppercase: false,
            }),
            Self::Relay(..) => TagKind::Relay,
            Self::POW { .. } => TagKind::Nonce,
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
            Self::Lnurl(..) => TagKind::Lnurl,
            Self::Url(..) => TagKind::Url,
            Self::MimeType(..) => TagKind::SingleLetter(SingleLetterTag {
                character: Alphabet::M,
                uppercase: false,
            }),
            Self::Aes256Gcm { .. } => TagKind::Aes256Gcm,
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
            Self::Label(..) => TagKind::SingleLetter(SingleLetterTag {
                character: Alphabet::L,
                uppercase: false,
            }),
            Self::Protected => TagKind::Protected,
            Self::Alt(..) => TagKind::Alt,
        }
    }

    /// Consume [`Tag`] and return string vector
    #[inline]
    pub fn to_vec(self) -> Vec<String> {
        self.into()
    }
}

impl From<TagStandard> for Vec<String> {
    fn from(tag: TagStandard) -> Self {
        let tag_kind: String = tag.kind().to_string();

        match tag {
            TagStandard::Event {
                event_id,
                relay_url,
                marker,
                public_key,
            } => {
                let mut tag = vec![tag_kind, event_id.to_hex()];
                if let Some(relay_url) = relay_url {
                    tag.push(relay_url.to_string());
                }
                if let Some(marker) = marker {
                    tag.resize_with(3, String::new);
                    tag.push(marker.to_string());
                }
                if let Some(public_key) = public_key {
                    tag.resize_with(4, String::new);
                    tag.push(public_key.to_string());
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
            } => {
                let mut vec = vec![tag_kind, coordinate.to_string()];
                if let Some(relay) = relay_url {
                    vec.push(relay.to_string());
                }
                vec
            }
            TagStandard::ExternalIdentity(identity) => identity.into(),
            TagStandard::Kind(kind) => vec![tag_kind, kind.to_string()],
            TagStandard::Relay(url) => vec![tag_kind, url.to_string()],
            TagStandard::POW { nonce, difficulty } => {
                vec![tag_kind, nonce.to_string(), difficulty.to_string()]
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
            TagStandard::Name(name) => {
                vec![tag_kind, name]
            }
            TagStandard::Lnurl(lnurl) => {
                vec![tag_kind, lnurl]
            }
            TagStandard::Url(url) => vec![tag_kind, url.to_string()],
            TagStandard::MimeType(mime) => vec![tag_kind, mime],
            TagStandard::Aes256Gcm { key, iv } => vec![tag_kind, key, iv],
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
            TagStandard::Label(l) => {
                let mut tag = Vec::with_capacity(1 + l.len());
                tag.push(tag_kind);
                tag.extend(l);
                tag
            }
            TagStandard::Protected => vec![tag_kind],
            TagStandard::Alt(summary) => vec![tag_kind, summary],
        }
    }
}

fn parse_a_tag<S>(tag: &[S]) -> Result<TagStandard, Error>
where
    S: AsRef<str>,
{
    if tag.len() >= 2 {
        let coordinate = Coordinate::from_str(tag[1].as_ref())?;
        let relay_url: Option<UncheckedUrl> = tag
            .get(2)
            .map(|r| r.as_ref())
            .and_then(|t| (!t.is_empty()).then_some(UncheckedUrl::from(t)));
        Ok(TagStandard::Coordinate {
            coordinate,
            relay_url,
        })
    } else {
        Err(Error::UnknownStardardizedTag)
    }
}

fn parse_e_tag<S>(tag: &[S]) -> Result<TagStandard, Error>
where
    S: AsRef<str>,
{
    if tag.len() >= 2 {
        let event_id: EventId = EventId::from_hex(tag[1].as_ref())?;

        let tag_2: Option<&str> = tag.get(2).map(|r| r.as_ref());
        let tag_3: Option<&str> = tag.get(3).map(|r| r.as_ref());
        let tag_4: Option<&str> = tag.get(4).map(|r| r.as_ref());

        // Check if it's a report
        if let Some(tag_2) = tag_2 {
            return match Report::from_str(tag_2) {
                Ok(report) => Ok(TagStandard::EventReport(event_id, report)),
                Err(_) => Ok(TagStandard::Event {
                    event_id,
                    relay_url: (!tag_2.is_empty()).then_some(UncheckedUrl::from(tag_2)),
                    marker: tag_3.and_then(|t| (!t.is_empty()).then_some(Marker::from(t))),
                    public_key: match tag_4 {
                        Some(public_key) => Some(PublicKey::from_hex(public_key)?),
                        None => None,
                    },
                }),
            };
        }

        Ok(TagStandard::event(event_id))
    } else {
        Err(Error::UnknownStardardizedTag)
    }
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
                relay_url: (!tag_2.is_empty()).then_some(UncheckedUrl::from(tag_2)),
                marker: LiveEventMarker::from_str(tag_3)?,
                proof: Signature::from_str(tag_4).ok(),
            });
        }

        if tag.len() >= 4 && !uppercase {
            let tag_2: &str = tag[2].as_ref();
            let tag_3: &str = tag[3].as_ref();

            let relay_url: Option<UncheckedUrl> =
                (!tag_2.is_empty()).then_some(UncheckedUrl::from(tag_2));

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
                        relay_url: Some(UncheckedUrl::from(tag_2)),
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
        Err(Error::UnknownStardardizedTag)
    }
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
        Err(Error::UnknownStardardizedTag)
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
