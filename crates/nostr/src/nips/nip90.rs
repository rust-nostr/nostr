// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP90: Data Vending Machines
//!
//! <https://github.com/nostr-protocol/nips/blob/master/90.md>

use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use core::fmt;
use core::num::ParseIntError;
use core::str::FromStr;

use super::util::{
    take_and_parse_from_str, take_and_parse_optional_relay_url, take_optional_string, take_string,
};
use crate::event::tag::{Tag, TagCodec, TagCodecError, impl_tag_codec_conversions};
use crate::types::url;
use crate::{Event, EventId, JsonUtil, PublicKey, RelayUrl, event};

const INPUT: &str = "i";
const OUTPUT: &str = "output";
const PARAM: &str = "param";
const BID: &str = "bid";
const RELAYS: &str = "relays";
const REQUEST: &str = "request";
const AMOUNT: &str = "amount";
const STATUS: &str = "status";
const ENCRYPTED: &str = "encrypted";

/// DVM Error
#[derive(Debug, PartialEq)]
pub enum Error {
    /// Event error
    Event(event::Error),
    /// Parse int error
    ParseInt(ParseIntError),
    /// Url error
    Url(url::Error),
    /// Codec error
    Codec(TagCodecError),
    /// Unknown input type
    UnknownInputType,
    /// Unknown status
    UnknownStatus,
}

impl core::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Event(e) => e.fmt(f),
            Self::ParseInt(e) => fmt::Display::fmt(e, f),
            Self::Url(e) => e.fmt(f),
            Self::Codec(e) => e.fmt(f),
            Self::UnknownInputType => f.write_str("Unknown input type"),
            Self::UnknownStatus => f.write_str("Unknown status"),
        }
    }
}

impl From<event::Error> for Error {
    fn from(e: event::Error) -> Self {
        Self::Event(e)
    }
}

impl From<ParseIntError> for Error {
    fn from(e: ParseIntError) -> Self {
        Self::ParseInt(e)
    }
}

impl From<url::Error> for Error {
    fn from(e: url::Error) -> Self {
        Self::Url(e)
    }
}

impl From<TagCodecError> for Error {
    fn from(e: TagCodecError) -> Self {
        Self::Codec(e)
    }
}

/// Data Vending Machine Status
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DataVendingMachineStatus {
    /// Service Provider requires payment before continuing
    PaymentRequired,
    /// Service Provider is processing the job
    Processing,
    /// Service Provider was unable to process the job
    Error,
    /// Service Provider successfully processed the job
    Success,
    /// Service Provider partially processed the job
    Partial,
}

impl fmt::Display for DataVendingMachineStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl DataVendingMachineStatus {
    /// Get as `&str`
    pub fn as_str(&self) -> &str {
        match self {
            Self::PaymentRequired => "payment-required",
            Self::Processing => "processing",
            Self::Error => "error",
            Self::Success => "success",
            Self::Partial => "partial",
        }
    }
}

impl FromStr for DataVendingMachineStatus {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "payment-required" => Ok(Self::PaymentRequired),
            "processing" => Ok(Self::Processing),
            "error" => Ok(Self::Error),
            "success" => Ok(Self::Success),
            "partial" => Ok(Self::Partial),
            _ => Err(Error::UnknownStatus),
        }
    }
}

/// Job input type
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum JobInputType {
    /// URL input
    Url,
    /// Event input
    Event,
    /// Previous job output input
    Job,
    /// Plain text input
    Text,
}

impl fmt::Display for JobInputType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl JobInputType {
    /// Get as `&str`
    pub fn as_str(&self) -> &str {
        match self {
            Self::Url => "url",
            Self::Event => "event",
            Self::Job => "job",
            Self::Text => "text",
        }
    }
}

impl FromStr for JobInputType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "url" => Ok(Self::Url),
            "event" => Ok(Self::Event),
            "job" => Ok(Self::Job),
            "text" => Ok(Self::Text),
            _ => Err(Error::UnknownInputType),
        }
    }
}

/// Standardized NIP-90 tags
///
/// <https://github.com/nostr-protocol/nips/blob/master/90.md>
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Nip90Tag {
    /// `i` tag
    Input {
        /// Input data
        data: String,
        /// Input type
        input_type: JobInputType,
        /// Relay hint for event/job inputs
        relay_hint: Option<RelayUrl>,
        /// Optional marker
        marker: Option<String>,
    },
    /// `output` tag
    Output(String),
    /// `param` tag
    Param {
        /// Parameter name
        name: String,
        /// Parameter value
        value: String,
    },
    /// `bid` tag
    Bid(u64),
    /// `relays` tag
    Relays(Vec<RelayUrl>),
    /// `request` tag
    Request(Event),
    /// `amount` tag
    Amount {
        /// Amount in millisats
        millisats: u64,
        /// Optional bolt11 invoice
        bolt11: Option<String>,
    },
    /// `status` tag
    Status {
        /// Job status
        status: DataVendingMachineStatus,
        /// Optional human-readable extra info
        extra_info: Option<String>,
    },
    /// `encrypted` tag
    Encrypted,
}

impl TagCodec for Nip90Tag {
    type Error = Error;

    fn parse<I, S>(tag: I) -> Result<Self, Self::Error>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut iter = tag.into_iter();
        let kind: S = iter.next().ok_or(TagCodecError::missing_tag_kind())?;

        match kind.as_ref() {
            INPUT => {
                let (data, input_type, relay_hint, marker) = parse_input_tag(iter)?;
                Ok(Self::Input {
                    data,
                    input_type,
                    relay_hint,
                    marker,
                })
            }
            OUTPUT => Ok(Self::Output(take_string(&mut iter, "output")?)),
            PARAM => Ok(Nip90Tag::Param {
                name: take_string(&mut iter, "param name")?,
                value: take_string(&mut iter, "param value")?,
            }),
            BID => {
                let bid: u64 = take_and_parse_from_str::<_, _, _, Error>(&mut iter, "bid")?;
                Ok(Self::Bid(bid))
            }
            RELAYS => Ok(Self::Relays(parse_relays_tag(iter)?)),
            REQUEST => {
                let request: S = iter.next().ok_or(TagCodecError::Missing("request"))?;
                Ok(Self::Request(Event::from_json(request.as_ref())?))
            }
            AMOUNT => {
                let (millisats, bolt11) = parse_amount_tag(iter)?;
                Ok(Self::Amount { millisats, bolt11 })
            }
            STATUS => {
                let (status, extra_info) = parse_status_tag(iter)?;
                Ok(Self::Status { status, extra_info })
            }
            ENCRYPTED => Ok(Self::Encrypted),
            _ => Err(TagCodecError::Unknown.into()),
        }
    }

    fn to_tag(&self) -> Tag {
        match self {
            Self::Input {
                data,
                input_type,
                relay_hint,
                marker,
            } => {
                let mut tag: Vec<String> = Vec::with_capacity(
                    3 + relay_hint.is_some() as usize + marker.is_some() as usize,
                );
                tag.push(String::from(INPUT));
                tag.push(data.clone());
                tag.push(input_type.to_string());

                match relay_hint {
                    Some(relay_hint) => tag.push(relay_hint.to_string()),
                    None => {
                        if marker.is_some() {
                            tag.push(String::new());
                        }
                    }
                }

                if let Some(marker) = marker {
                    tag.push(marker.clone());
                }

                Tag::new(tag)
            }
            Self::Output(output) => Tag::new(vec![String::from(OUTPUT), output.clone()]),
            Self::Param { name, value } => {
                Tag::new(vec![String::from(PARAM), name.clone(), value.clone()])
            }
            Self::Bid(bid) => Tag::new(vec![String::from(BID), bid.to_string()]),
            Self::Relays(relays) => {
                let mut tag: Vec<String> = Vec::with_capacity(relays.len() + 1);
                tag.push(String::from(RELAYS));
                tag.extend(relays.iter().map(ToString::to_string));
                Tag::new(tag)
            }
            Self::Request(event) => Tag::new(vec![String::from(REQUEST), event.as_json()]),
            Self::Amount { millisats, bolt11 } => {
                let mut tag: Vec<String> = vec![String::from(AMOUNT), millisats.to_string()];
                if let Some(bolt11) = bolt11 {
                    tag.push(bolt11.clone());
                }
                Tag::new(tag)
            }
            Self::Status { status, extra_info } => {
                let mut tag: Vec<String> = vec![String::from(STATUS), status.to_string()];
                if let Some(extra_info) = extra_info {
                    tag.push(extra_info.clone());
                }
                Tag::new(tag)
            }
            Self::Encrypted => Tag::new(vec![String::from(ENCRYPTED)]),
        }
    }
}

impl_tag_codec_conversions!(Nip90Tag);

fn parse_input_tag<T, S>(
    mut iter: T,
) -> Result<(String, JobInputType, Option<RelayUrl>, Option<String>), Error>
where
    T: Iterator<Item = S>,
    S: AsRef<str>,
{
    let data: String = take_string(&mut iter, "input data")?;
    let input_type: JobInputType =
        take_and_parse_from_str::<_, _, _, Error>(&mut iter, "input type")?;
    let relay_hint: Option<RelayUrl> = take_and_parse_optional_relay_url(&mut iter)?;
    let marker: Option<String> = take_optional_string(&mut iter);

    Ok((data, input_type, relay_hint, marker))
}

fn parse_relays_tag<T, S>(iter: T) -> Result<Vec<RelayUrl>, Error>
where
    T: Iterator<Item = S>,
    S: AsRef<str>,
{
    let mut relays: Vec<RelayUrl> = Vec::new();
    for relay in iter {
        relays.push(RelayUrl::parse(relay.as_ref())?);
    }
    Ok(relays)
}

fn parse_amount_tag<T, S>(mut iter: T) -> Result<(u64, Option<String>), Error>
where
    T: Iterator<Item = S>,
    S: AsRef<str>,
{
    let millisats: u64 = take_and_parse_from_str::<_, _, _, Error>(&mut iter, "amount")?;
    let bolt11: Option<String> = take_optional_string(&mut iter);

    Ok((millisats, bolt11))
}

fn parse_status_tag<T, S>(mut iter: T) -> Result<(DataVendingMachineStatus, Option<String>), Error>
where
    T: Iterator<Item = S>,
    S: AsRef<str>,
{
    let status: DataVendingMachineStatus =
        take_and_parse_from_str::<_, _, _, Error>(&mut iter, "status")?;
    let extra_info: Option<String> = take_optional_string(&mut iter);

    Ok((status, extra_info))
}

/// Data Vending Machine (DVM) - Job Feedback data
///
/// <https://github.com/nostr-protocol/nips/blob/master/90.md>
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct JobFeedbackData {
    pub(crate) job_request_id: EventId,
    pub(crate) customer_public_key: PublicKey,
    pub(crate) status: DataVendingMachineStatus,
    pub(crate) extra_info: Option<String>,
    pub(crate) amount_msat: Option<u64>,
    pub(crate) bolt11: Option<String>,
    pub(crate) payload: Option<String>,
}

impl JobFeedbackData {
    /// Construct new Job Feedback
    pub fn new(job_request: &Event, status: DataVendingMachineStatus) -> Self {
        Self {
            job_request_id: job_request.id,
            customer_public_key: job_request.pubkey,
            status,
            extra_info: None,
            amount_msat: None,
            bolt11: None,
            payload: None,
        }
    }

    /// Add extra info
    #[inline]
    pub fn extra_info<S>(mut self, info: S) -> Self
    where
        S: Into<String>,
    {
        self.extra_info = Some(info.into());
        self
    }

    /// Add payment amount
    #[inline]
    pub fn amount(mut self, millisats: u64, bolt11: Option<String>) -> Self {
        self.amount_msat = Some(millisats);
        self.bolt11 = bolt11;
        self
    }

    /// Add payload
    #[inline]
    pub fn payload<S>(mut self, payload: S) -> Self
    where
        S: Into<String>,
    {
        self.payload = Some(payload.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_input_tag() {
        let tag = vec!["i", "hello", "text"];
        let parsed = Nip90Tag::parse(&tag).unwrap();
        assert_eq!(
            parsed,
            Nip90Tag::Input {
                data: String::from("hello"),
                input_type: JobInputType::Text,
                relay_hint: None,
                marker: None,
            }
        );
        assert_eq!(parsed.to_tag(), Tag::parse(tag).unwrap());
    }

    #[test]
    fn test_parse_param_tag() {
        let tag = vec!["param", "lang", "es"];
        let parsed = Nip90Tag::parse(&tag).unwrap();
        assert_eq!(
            parsed,
            Nip90Tag::Param {
                name: String::from("lang"),
                value: String::from("es"),
            }
        );
        assert_eq!(parsed.to_tag(), Tag::parse(tag).unwrap());
    }

    #[test]
    fn test_parse_amount_tag() {
        let tag = vec!["amount", "21000", "lnbc1..."];
        let parsed = Nip90Tag::parse(&tag).unwrap();
        assert_eq!(
            parsed,
            Nip90Tag::Amount {
                millisats: 21000,
                bolt11: Some(String::from("lnbc1...")),
            }
        );
        assert_eq!(parsed.to_tag(), Tag::parse(tag).unwrap());
    }

    #[test]
    fn test_parse_status_tag() {
        let tag = vec!["status", "processing", "working"];
        let parsed = Nip90Tag::parse(&tag).unwrap();
        assert_eq!(
            parsed,
            Nip90Tag::Status {
                status: DataVendingMachineStatus::Processing,
                extra_info: Some(String::from("working")),
            }
        );
        assert_eq!(parsed.to_tag(), Tag::parse(tag).unwrap());
    }

    #[test]
    fn test_parse_encrypted_tag() {
        let tag = vec!["encrypted"];
        let parsed = Nip90Tag::parse(&tag).unwrap();
        assert_eq!(parsed, Nip90Tag::Encrypted);
        assert_eq!(parsed.to_tag(), Tag::parse(tag).unwrap());
    }
}
