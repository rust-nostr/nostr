// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP-56: Reporting
//!
//! <https://github.com/nostr-protocol/nips/blob/master/56.md>

use alloc::string::{String, ToString};
use alloc::vec;
use core::fmt;
use core::str::FromStr;

use super::util::{take_and_parse_from_str, take_event_id, take_public_key};
use crate::event::tag::{Tag, TagCodec, TagCodecError, impl_tag_codec_conversions};
use crate::{EventId, PublicKey, event, key};

/// NIP56 error
#[derive(Debug, PartialEq)]
pub enum Error {
    /// Keys error
    Keys(key::Error),
    /// Event error
    Event(event::Error),
    /// Codec error
    Codec(TagCodecError),
    /// Unknown [`Report`]
    UnknownReportType,
}

impl core::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Keys(e) => e.fmt(f),
            Self::Event(e) => e.fmt(f),
            Self::Codec(e) => e.fmt(f),
            Self::UnknownReportType => f.write_str("Unknown report type"),
        }
    }
}

impl From<key::Error> for Error {
    fn from(e: key::Error) -> Self {
        Self::Keys(e)
    }
}

impl From<event::Error> for Error {
    fn from(e: event::Error) -> Self {
        Self::Event(e)
    }
}

impl From<TagCodecError> for Error {
    fn from(e: TagCodecError) -> Self {
        Self::Codec(e)
    }
}

/// Report
///
/// <https://github.com/nostr-protocol/nips/blob/master/56.md>
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Report {
    /// Depictions of nudity, porn, etc
    Nudity,
    /// Virus, trojan horse, worm, robot, spyware, adware, back door, ransomware, rootkit, kidnapper, etc.
    Malware,
    /// Profanity, hateful speech, etc.
    Profanity,
    /// Something which may be illegal in some jurisdiction
    Illegal,
    /// Spam
    Spam,
    /// Someone pretending to be someone else
    Impersonation,
    ///  Reports that don't fit in the above categories
    Other,
}

impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Report {
    /// Get as `&str`
    pub fn as_str(&self) -> &str {
        match self {
            Self::Nudity => "nudity",
            Self::Malware => "malware",
            Self::Profanity => "profanity",
            Self::Illegal => "illegal",
            Self::Spam => "spam",
            Self::Impersonation => "impersonation",
            Self::Other => "other",
        }
    }
}

impl FromStr for Report {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "nudity" => Ok(Self::Nudity),
            "malware" => Ok(Self::Malware),
            "profanity" => Ok(Self::Profanity),
            "illegal" => Ok(Self::Illegal),
            "spam" => Ok(Self::Spam),
            "impersonation" => Ok(Self::Impersonation),
            "other" => Ok(Self::Other),
            _ => Err(Error::UnknownReportType),
        }
    }
}

/// Standardized NIP-56 tags
///
/// <https://github.com/nostr-protocol/nips/blob/master/56.md>
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Nip56Tag {
    /// `e` tag
    Event {
        /// Event ID
        id: EventId,
        /// Report
        report: Report,
    },
    /// `p` tag
    PublicKey {
        /// Public key
        public_key: PublicKey,
        /// Report
        report: Report,
    },
}

impl TagCodec for Nip56Tag {
    type Error = Error;

    fn parse<I, S>(tag: I) -> Result<Self, Self::Error>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        // Take iterator
        let mut iter = tag.into_iter();

        // Extract first value
        let kind: S = iter.next().ok_or(TagCodecError::missing_tag_kind())?;

        // Match kind
        match kind.as_ref() {
            "e" => {
                let (id, report) = parse_e_tag(iter)?;
                Ok(Self::Event { id, report })
            }
            "p" => {
                let (public_key, report) = parse_p_tag(iter)?;
                Ok(Self::PublicKey { public_key, report })
            }
            _ => Err(TagCodecError::Unknown.into()),
        }
    }

    fn to_tag(&self) -> Tag {
        match self {
            Self::Event { id, report } => {
                Tag::new(vec![String::from("e"), id.to_hex(), report.to_string()])
            }
            Self::PublicKey { public_key, report } => Tag::new(vec![
                String::from("p"),
                public_key.to_hex(),
                report.to_string(),
            ]),
        }
    }
}

impl_tag_codec_conversions!(Nip56Tag);

fn parse_e_tag<T, S>(mut iter: T) -> Result<(EventId, Report), Error>
where
    T: Iterator<Item = S>,
    S: AsRef<str>,
{
    let id: EventId = take_event_id::<_, _, Error>(&mut iter)?;
    let report: Report = take_and_parse_from_str::<_, _, _, Error>(&mut iter, "report")?;

    Ok((id, report))
}

fn parse_p_tag<T, S>(mut iter: T) -> Result<(PublicKey, Report), Error>
where
    T: Iterator<Item = S>,
    S: AsRef<str>,
{
    let public_key: PublicKey = take_public_key::<_, _, Error>(&mut iter)?;
    let report: Report = take_and_parse_from_str::<_, _, _, Error>(&mut iter, "report")?;

    Ok((public_key, report))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_report_e_tag() {
        let tag = vec![
            "e",
            "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7",
            "nudity",
        ];
        let parsed = Nip56Tag::parse(&tag).unwrap();

        assert_eq!(
            parsed,
            Nip56Tag::Event {
                id: EventId::from_hex(
                    "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7"
                )
                .unwrap(),
                report: Report::Nudity,
            }
        );
        assert_eq!(parsed.to_tag(), Tag::parse(tag).unwrap());
    }

    #[test]
    fn test_report_p_tag() {
        let tag = vec![
            "p",
            "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d",
            "impersonation",
        ];
        let parsed = Nip56Tag::parse(&tag).unwrap();

        assert_eq!(
            parsed,
            Nip56Tag::PublicKey {
                public_key: PublicKey::from_hex(
                    "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
                )
                .unwrap(),
                report: Report::Impersonation,
            }
        );
        assert_eq!(parsed.to_tag(), Tag::parse(tag).unwrap());
    }

    #[test]
    fn test_missing_report() {
        let tag = vec![
            "p",
            "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d",
        ];
        assert!(matches!(
            Nip56Tag::parse(&tag).unwrap_err(),
            Error::Codec(TagCodecError::Missing("report"))
        ));

        let tag = vec![
            "e",
            "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7",
        ];
        assert!(matches!(
            Nip56Tag::parse(&tag).unwrap_err(),
            Error::Codec(TagCodecError::Missing("report"))
        ));
    }

    #[test]
    fn test_empty_report() {
        let tag = vec![
            "p",
            "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d",
            "",
        ];
        assert!(matches!(
            Nip56Tag::parse(&tag).unwrap_err(),
            Error::UnknownReportType
        ));

        let tag = vec![
            "e",
            "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7",
            "",
        ];
        assert!(matches!(
            Nip56Tag::parse(&tag).unwrap_err(),
            Error::UnknownReportType
        ));
    }
}
