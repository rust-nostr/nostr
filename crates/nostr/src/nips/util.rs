use alloc::string::{String, ToString};
use core::num::ParseIntError;
use core::str::FromStr;

use super::nip01::{self, Coordinate};
use crate::event::tag::TagCodecError;
use crate::event::{self, EventId};
use crate::key::{self, PublicKey};
use crate::types::time::Timestamp;
use crate::types::url::{self, RelayUrl};

#[inline]
pub(super) fn take_and_parse_optional_public_key<I, S>(
    iter: &mut I,
) -> Result<Option<PublicKey>, key::Error>
where
    I: Iterator<Item = S>,
    S: AsRef<str>,
{
    take_and_parse_optional(iter, PublicKey::from_hex)
}

#[inline]
pub(super) fn take_and_parse_optional_coordinate<I, S>(
    iter: &mut I,
) -> Result<Option<Coordinate>, nip01::Error>
where
    I: Iterator<Item = S>,
    S: AsRef<str>,
{
    take_and_parse_optional(iter, Coordinate::from_kpi_format)
}

#[inline]
pub(super) fn take_and_parse_optional_relay_url<I, S>(
    iter: &mut I,
) -> Result<Option<RelayUrl>, url::Error>
where
    I: Iterator<Item = S>,
    S: AsRef<str>,
{
    take_and_parse_optional(iter, RelayUrl::parse)
}

#[inline]
pub(super) fn take_and_parse_optional_from_str<I, S, T>(iter: &mut I) -> Result<Option<T>, T::Err>
where
    I: Iterator<Item = S>,
    S: AsRef<str>,
    T: FromStr,
{
    take_and_parse_optional(iter, T::from_str)
}

/// Take and parse an **optional** value with the provided parser.
///
/// If the value is missing or empty, `None` is returned.
fn take_and_parse_optional<I, S, T, E>(
    iter: &mut I,
    parse: impl FnOnce(&str) -> Result<T, E>,
) -> Result<Option<T>, E>
where
    I: Iterator<Item = S>,
    S: AsRef<str>,
{
    match iter.next() {
        Some(value) => {
            let value: &str = value.as_ref();

            if value.is_empty() {
                Ok(None)
            } else {
                // NOTE: we don't use FromStr::from_str here because some implementations, like PublicKey::from_str, support parsing both of hex and also bech32 or URIs, but tags must use just hex.
                parse(value).map(Some)
            }
        }
        None => Ok(None),
    }
}

/// Take an **optional** string
///
/// If the value is empty, None is returned.
pub(super) fn take_optional_string<I, S>(iter: &mut I) -> Option<String>
where
    I: Iterator<Item = S>,
    S: AsRef<str>,
{
    iter.next().and_then(|value| {
        let value: &str = value.as_ref();

        if value.is_empty() {
            None
        } else {
            Some(value.to_string())
        }
    })
}

/// Take a string
pub(super) fn take_string<I, S>(
    iter: &mut I,
    missing_error: &'static str,
) -> Result<String, TagCodecError>
where
    I: Iterator<Item = S>,
    S: AsRef<str>,
{
    let value: S = iter.next().ok_or(TagCodecError::Missing(missing_error))?;
    Ok(value.as_ref().to_string())
}

pub(super) fn take_public_key<I, S, E>(iter: &mut I) -> Result<PublicKey, E>
where
    I: Iterator<Item = S>,
    S: AsRef<str>,
    E: From<key::Error> + From<TagCodecError>,
{
    let public_key: S = iter.next().ok_or(TagCodecError::Missing("public key"))?;
    let public_key: PublicKey = PublicKey::from_hex(public_key.as_ref())?;
    Ok(public_key)
}

pub(super) fn take_coordinate<I, S, E>(iter: &mut I) -> Result<Coordinate, E>
where
    I: Iterator<Item = S>,
    S: AsRef<str>,
    E: From<nip01::Error> + From<TagCodecError>,
{
    let coordinate: S = iter.next().ok_or(TagCodecError::Missing("coordinate"))?;
    let coordinate: Coordinate = Coordinate::from_kpi_format(coordinate.as_ref())?;
    Ok(coordinate)
}

pub(super) fn take_event_id<I, S, E>(iter: &mut I) -> Result<EventId, E>
where
    I: Iterator<Item = S>,
    S: AsRef<str>,
    E: From<event::Error> + From<TagCodecError>,
{
    let event_id: S = iter.next().ok_or(TagCodecError::Missing("event ID"))?;
    let event_id: EventId = EventId::from_hex(event_id.as_ref())?;
    Ok(event_id)
}

#[inline]
pub(super) fn take_relay_url<T, S, E>(iter: &mut T) -> Result<RelayUrl, E>
where
    T: Iterator<Item = S>,
    S: AsRef<str>,
    E: From<url::Error> + From<TagCodecError>,
{
    take_and_parse_from_str(iter, "relay URL")
}

#[inline]
pub(super) fn take_timestamp<T, S, E>(iter: &mut T) -> Result<Timestamp, E>
where
    T: Iterator<Item = S>,
    S: AsRef<str>,
    E: From<ParseIntError> + From<TagCodecError>,
{
    take_and_parse_from_str(iter, "timestamp")
}

pub(super) fn take_and_parse_from_str<O, T, S, E>(
    iter: &mut T,
    missing_error: &'static str,
) -> Result<O, E>
where
    T: Iterator<Item = S>,
    S: AsRef<str>,
    O: FromStr,
    E: From<O::Err> + From<TagCodecError>,
{
    let value: S = iter.next().ok_or(TagCodecError::Missing(missing_error))?;
    let value: O = O::from_str(value.as_ref())?;
    Ok(value)
}
