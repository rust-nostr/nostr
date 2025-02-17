// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP21: `nostr:` URI scheme
//!
//! <https://github.com/nostr-protocol/nips/blob/master/21.md>

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;
use core::str::FromStr;

use bech32::Fe32;

use super::nip19::{
    self, FromBech32, Nip19, Nip19Coordinate, Nip19Event, Nip19Prefix, Nip19Profile, ToBech32,
};
use crate::nips::nip01::Coordinate;
use crate::{EventId, PublicKey};

/// URI scheme
pub const SCHEME: &str = "nostr";
const SCHEME_WITH_COLON: &str = "nostr:";

/// Unsupported Bech32 Type
#[derive(Debug, PartialEq, Eq)]
pub enum UnsupportedVariant {
    /// Secret Key
    SecretKey,
}

impl fmt::Display for UnsupportedVariant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SecretKey => write!(f, "secret key"),
        }
    }
}

/// NIP21 error
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// NIP19 error
    NIP19(nip19::Error),
    /// Unsupported bech32 type
    UnsupportedVariant(UnsupportedVariant),
    /// Invalid nostr URI
    InvalidURI,
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NIP19(e) => write!(f, "NIP19: {e}"),
            Self::UnsupportedVariant(t) => write!(f, "Unsupported variant: {t}"),
            Self::InvalidURI => write!(f, "Invalid nostr URI"),
        }
    }
}

impl From<nip19::Error> for Error {
    fn from(e: nip19::Error) -> Self {
        Self::NIP19(e)
    }
}

fn split_uri(uri: &str) -> Result<&str, Error> {
    let mut splitted = uri.split(':');
    let prefix: &str = splitted.next().ok_or(Error::InvalidURI)?;

    if prefix != SCHEME {
        return Err(Error::InvalidURI);
    }

    splitted.next().ok_or(Error::InvalidURI)
}

/// To nostr URI trait
pub trait ToNostrUri: ToBech32
where
    Error: From<<Self as ToBech32>::Err>,
{
    /// Get nostr URI
    #[inline]
    fn to_nostr_uri(&self) -> Result<String, Error> {
        Ok(format!("{SCHEME}:{}", self.to_bech32()?))
    }
}

/// From nostr URI trait
pub trait FromNostrUri: FromBech32
where
    Error: From<<Self as FromBech32>::Err>,
{
    /// From `nostr:` URI
    #[inline]
    fn from_nostr_uri(uri: &str) -> Result<Self, Error> {
        let data: &str = split_uri(uri)?;
        Ok(Self::from_bech32(data)?)
    }
}

impl ToNostrUri for PublicKey {}
impl FromNostrUri for PublicKey {}
impl ToNostrUri for EventId {}
impl FromNostrUri for EventId {}
impl ToNostrUri for Nip19Profile {}
impl FromNostrUri for Nip19Profile {}
impl ToNostrUri for Nip19Event {}
impl FromNostrUri for Nip19Event {}
impl FromNostrUri for Coordinate {}
impl ToNostrUri for Nip19Coordinate {}
impl FromNostrUri for Nip19Coordinate {}

/// A representation any `NIP21` object. Useful for decoding
/// `NIP21` strings without necessarily knowing what you're decoding
/// ahead of time.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Nip21 {
    /// nostr::npub
    Pubkey(PublicKey),
    /// nostr::nprofile
    Profile(Nip19Profile),
    /// nostr::note
    EventId(EventId),
    /// nostr::nevent
    Event(Nip19Event),
    /// nostr::naddr
    Coordinate(Nip19Coordinate),
}

impl From<Nip21> for Nip19 {
    fn from(value: Nip21) -> Self {
        match value {
            Nip21::Pubkey(val) => Self::Pubkey(val),
            Nip21::Profile(val) => Self::Profile(val),
            Nip21::EventId(val) => Self::EventId(val),
            Nip21::Event(val) => Self::Event(val),
            Nip21::Coordinate(val) => Self::Coordinate(val),
        }
    }
}

impl TryFrom<Nip19> for Nip21 {
    type Error = Error;

    fn try_from(value: Nip19) -> Result<Self, Self::Error> {
        match value {
            Nip19::Secret(..) => Err(Error::UnsupportedVariant(UnsupportedVariant::SecretKey)),
            #[cfg(feature = "nip49")]
            Nip19::EncryptedSecret(..) => {
                Err(Error::UnsupportedVariant(UnsupportedVariant::SecretKey))
            }
            Nip19::Pubkey(val) => Ok(Self::Pubkey(val)),
            Nip19::Profile(val) => Ok(Self::Profile(val)),
            Nip19::EventId(val) => Ok(Self::EventId(val)),
            Nip19::Event(val) => Ok(Self::Event(val)),
            Nip19::Coordinate(val) => Ok(Self::Coordinate(val)),
        }
    }
}

impl Nip21 {
    /// Parse NIP21 string
    #[inline]
    pub fn parse<S>(uri: S) -> Result<Self, Error>
    where
        S: AsRef<str>,
    {
        let data: &str = split_uri(uri.as_ref())?;
        let nip19: Nip19 = Nip19::from_bech32(data)?;
        Self::try_from(nip19)
    }

    /// Serialize to NIP21 nostr URI
    pub fn to_nostr_uri(&self) -> Result<String, Error> {
        match self {
            Self::Pubkey(val) => Ok(val.to_nostr_uri()?),
            Self::Profile(val) => Ok(val.to_nostr_uri()?),
            Self::EventId(val) => Ok(val.to_nostr_uri()?),
            Self::Event(val) => Ok(val.to_nostr_uri()?),
            Self::Coordinate(val) => Ok(val.to_nostr_uri()?),
        }
    }

    /// Get [EventId] if exists
    pub fn event_id(&self) -> Option<EventId> {
        match self {
            Self::EventId(id) => Some(*id),
            Self::Event(e) => Some(e.event_id),
            _ => None,
        }
    }
}

/// Extract `nostr:` URIs from a text
///
/// The returned vector maintains the same order as their occurrences in the input.
///
/// The result **is not deduplicated**, meaning that if a URI appears multiple times in the input,
/// it will appear the same number of times in the output.
pub fn extract_from_text(content: &str) -> Vec<Nip21> {
    let mut list: Vec<Nip21> = Vec::new();

    // Split `nostr:` strings
    let mut splitted = content.split(SCHEME_WITH_COLON);

    // Remove the first value.
    // The first value will never be a nostr URI.
    splitted.next();

    for val in splitted.filter(|v| !v.is_empty()) {
        // Parse prefix from string
        match Nip19Prefix::from_str(val) {
            Ok(prefix) => {
                // Now we have the following string:
                // <bech32-prefix><bech32-data>[<non-bech32-data>]
                // We need to take the valid bech32 string, so `<bech32-prefix>` + `<bech32-data>`.

                // Take the valid bech32 string
                if let Some(valid) = take_valid_bech32(prefix, val) {
                    // Try to parse the valid bech32 string as NIP19
                    if let Ok(val) = Nip19::from_bech32(valid) {
                        // Try to convert NIP19 to NIP21
                        if let Ok(item) = Nip21::try_from(val) {
                            list.push(item);
                        }
                    }
                }
            }
            Err(..) => continue,
        }
    }

    list
}

fn take_valid_bech32(prefix: Nip19Prefix, full_input: &str) -> Option<&str> {
    // <prefix> + 1
    // Example: npub1, nsec1, nevent1
    let full_prefix_len: usize = prefix.len() + 1;

    // Get input without prefix
    let input: &str = full_input.get(full_prefix_len..)?;

    // Keep track of valid len
    let mut valid_len: usize = 0;

    for c in input.chars() {
        // Check if it's a valid bech32 char
        match Fe32::from_char(c) {
            Ok(..) => {
                // The UTF8 len for bech32 chars will always be `1`
                valid_len += 1;
            }
            Err(..) => break,
        }
    }

    // Take full valid input
    full_input.get(..full_prefix_len + valid_len)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_nostr_uri() {
        let pubkey =
            PublicKey::from_str("aa4fc8665f5696e33db7e1a572e3b0f5b3d615837b0f362dcb1c8068b098c7b4")
                .unwrap();
        assert_eq!(
            pubkey.to_nostr_uri().unwrap(),
            String::from("nostr:npub14f8usejl26twx0dhuxjh9cas7keav9vr0v8nvtwtrjqx3vycc76qqh9nsy")
        );

        let generic = Nip21::Pubkey(pubkey);
        assert_eq!(
            generic.to_nostr_uri().unwrap(),
            String::from("nostr:npub14f8usejl26twx0dhuxjh9cas7keav9vr0v8nvtwtrjqx3vycc76qqh9nsy")
        );
    }

    #[test]
    fn test_from_nostr_uri() {
        let pubkey =
            PublicKey::from_str("aa4fc8665f5696e33db7e1a572e3b0f5b3d615837b0f362dcb1c8068b098c7b4")
                .unwrap();
        assert_eq!(
            PublicKey::from_nostr_uri(
                "nostr:npub14f8usejl26twx0dhuxjh9cas7keav9vr0v8nvtwtrjqx3vycc76qqh9nsy"
            )
            .unwrap(),
            pubkey
        );

        assert_eq!(
            Nip21::parse("nostr:npub14f8usejl26twx0dhuxjh9cas7keav9vr0v8nvtwtrjqx3vycc76qqh9nsy")
                .unwrap(),
            Nip21::Pubkey(pubkey),
        );

        assert_eq!(
            Nip21::parse("nostr:nprofile1qqsr9cvzwc652r4m83d86ykplrnm9dg5gwdvzzn8ameanlvut35wy3gpz4mhxue69uhhyetvv9ujuerpd46hxtnfduhsz4nxck").unwrap(),
            Nip21::Profile(Nip19Profile::new(
                PublicKey::from_str(
                    "32e1827635450ebb3c5a7d12c1f8e7b2b514439ac10a67eef3d9fd9c5c68e245",
                )
                .unwrap(),
                ["wss://relay.damus.io/"]
            ).unwrap()),
        );
    }

    #[test]
    fn test_unsupported_from_nostr_uri() {
        assert_eq!(
            Nip21::parse("nostr:nsec1j4c6269y9w0q2er2xjw8sv2ehyrtfxq3jwgdlxj6qfn8z4gjsq5qfvfk99")
                .unwrap_err(),
            Error::UnsupportedVariant(UnsupportedVariant::SecretKey)
        );
    }

    #[test]
    fn test_extract_nostr_uris_from_text() {
        let pubkey = Nip21::Pubkey(
            PublicKey::from_bech32(
                "npub1drvpzev3syqt0kjrls50050uzf25gehpz9vgdw08hvex7e0vgfeq0eseet",
            )
            .unwrap(),
        );
        let pubkey2 = Nip21::Pubkey(
            PublicKey::from_bech32(
                "npub1acg6thl5psv62405rljzkj8spesceyfz2c32udakc2ak0dmvfeyse9p35c",
            )
            .unwrap(),
        );
        let event = Nip21::Event(Nip19Event::from_bech32("nevent1qqsz8xjlh82ykfr3swjk5fw0l3v33pcsaq4z6f7q0zy2dxrfm7x2yeqpz4mhxue69uhkummnw3ezummcw3ezuer9wchsygrgmqgktyvpqzma5slu9rmarlqj24zxdcg3tzrtneamxfhktmzzwgpsgqqqqqqsmxphku").unwrap());
        let profile1 = Nip21::Profile(Nip19Profile::from_bech32("nprofile1qqsqfyvdlsmvj0nakmxq6c8n0c2j9uwrddjd8a95ynzn9479jhlth3gpvemhxue69uhkv6tvw3jhytnwdaehgu3wwa5kuef0dec82c33w94xwcmdd3cxketedsux6ertwecrgues0pk8xdrew33h27pkd4unvvpkw3nkv7pe0p68gat58ycrw6ps0fenwdnvva48w0mzwfhkzerrv9ehg0t5wf6k2qgnwaehxw309ac82unsd3jhqct89ejhxtcpz4mhxue69uhhyetvv9ujuerpd46hxtnfduhsh8njvk").unwrap());
        let profile2 = Nip21::Profile(Nip19Profile::from_bech32("nprofile1qqswuyd9ml6qcxd92h6pleptfrcqucvvjy39vg4wx7mv9wm8kakyujgpypmhxue69uhkx6r0wf6hxtndd94k2erfd3nk2u3wvdhk6w35xs6z7qgwwaehxw309ahx7uewd3hkctcpypmhxue69uhkummnw3ezuetfde6kuer6wasku7nfvuh8xurpvdjj7a0nq40").unwrap());

        let vector = vec![
            ("#rustnostr #nostr #kotlin #jvm\n\nnostr:nevent1qqsz8xjlh82ykfr3swjk5fw0l3v33pcsaq4z6f7q0zy2dxrfm7x2yeqpz4mhxue69uhkummnw3ezummcw3ezuer9wchsygrgmqgktyvpqzma5slu9rmarlqj24zxdcg3tzrtneamxfhktmzzwgpsgqqqqqqsmxphku", vec![event]),
            ("I have never been very active in discussions but working on rust-nostr (at the time called nostr-rs-sdk) since September 2022 🦀 \n\nIf I remember correctly there were also nostr:nprofile1qqsqfyvdlsmvj0nakmxq6c8n0c2j9uwrddjd8a95ynzn9479jhlth3gpvemhxue69uhkv6tvw3jhytnwdaehgu3wwa5kuef0dec82c33w94xwcmdd3cxketedsux6ertwecrgues0pk8xdrew33h27pkd4unvvpkw3nkv7pe0p68gat58ycrw6ps0fenwdnvva48w0mzwfhkzerrv9ehg0t5wf6k2qgnwaehxw309ac82unsd3jhqct89ejhxtcpz4mhxue69uhhyetvv9ujuerpd46hxtnfduhsh8njvk and nostr:nprofile1qqswuyd9ml6qcxd92h6pleptfrcqucvvjy39vg4wx7mv9wm8kakyujgpypmhxue69uhkx6r0wf6hxtndd94k2erfd3nk2u3wvdhk6w35xs6z7qgwwaehxw309ahx7uewd3hkctcpypmhxue69uhkummnw3ezuetfde6kuer6wasku7nfvuh8xurpvdjj7a0nq40", vec![profile1, profile2.clone()]),
            ("Test ending with full stop: nostr:npub1drvpzev3syqt0kjrls50050uzf25gehpz9vgdw08hvex7e0vgfeq0eseet.", vec![pubkey.clone()]),
            ("nostr:npub1drvpzev3syqt0kjrls50050uzf25gehpz9vgdw08hvex7e0vgfeq0eseet", vec![pubkey.clone()]),
            ("Public key without prefix npub1drvpzev3syqt0kjrls50050uzf25gehpz9vgdw08hvex7e0vgfeq0eseet", vec![]),
            ("Public key `nostr:npub1drvpzev3syqt0kjrls50050uzf25gehpz9vgdw08hvex7e0vgfeq0eseet+.", vec![pubkey.clone()]),
            ("Duplicated npub: nostr:npub1drvpzev3syqt0kjrls50050uzf25gehpz9vgdw08hvex7e0vgfeq0eseet, nostr:npub1drvpzev3syqt0kjrls50050uzf25gehpz9vgdw08hvex7e0vgfeq0eseet", vec![pubkey.clone(), pubkey]),
            ("Uppercase nostr:npub1DRVpZev3syqt0kjrls50050uzf25gehpz9vgdw08hvex7e0vgfeq0eSEET", vec![]),
            ("Npub and nprofile that point to the same public key: nostr:npub1acg6thl5psv62405rljzkj8spesceyfz2c32udakc2ak0dmvfeyse9p35c and nostr:nprofile1qqswuyd9ml6qcxd92h6pleptfrcqucvvjy39vg4wx7mv9wm8kakyujgpypmhxue69uhkx6r0wf6hxtndd94k2erfd3nk2u3wvdhk6w35xs6z7qgwwaehxw309ahx7uewd3hkctcpypmhxue69uhkummnw3ezuetfde6kuer6wasku7nfvuh8xurpvdjj7a0nq40", vec![pubkey2, profile2]),
            ("content without nostr URIs", vec![]),
        ];

        for (content, expected) in vector {
            let objs = extract_from_text(content);
            assert_eq!(objs, expected);
        }
    }
}

#[cfg(bench)]
mod benches {
    use test::{black_box, Bencher};

    use super::*;

    #[bench]
    pub fn extract_nostr_uris_from_text(bh: &mut Bencher) {
        let text: &str = "I have never been very active in discussions but working on rust-nostr (at the time called nostr-rs-sdk) since September 2022 🦀 \n\nIf I remember correctly there were also nostr:nprofile1qqsqfyvdlsmvj0nakmxq6c8n0c2j9uwrddjd8a95ynzn9479jhlth3gpvemhxue69uhkv6tvw3jhytnwdaehgu3wwa5kuef0dec82c33w94xwcmdd3cxketedsux6ertwecrgues0pk8xdrew33h27pkd4unvvpkw3nkv7pe0p68gat58ycrw6ps0fenwdnvva48w0mzwfhkzerrv9ehg0t5wf6k2qgnwaehxw309ac82unsd3jhqct89ejhxtcpz4mhxue69uhhyetvv9ujuerpd46hxtnfduhsh8njvk and nostr:nprofile1qqswuyd9ml6qcxd92h6pleptfrcqucvvjy39vg4wx7mv9wm8kakyujgpypmhxue69uhkx6r0wf6hxtndd94k2erfd3nk2u3wvdhk6w35xs6z7qgwwaehxw309ahx7uewd3hkctcpypmhxue69uhkummnw3ezuetfde6kuer6wasku7nfvuh8xurpvdjj7a0nq40";
        bh.iter(|| {
            black_box(extract_from_text(text));
        });
    }
}
