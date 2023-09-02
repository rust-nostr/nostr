// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! NIP33
//!
//! <https://github.com/nostr-protocol/nips/blob/master/33.md>

use alloc::string::{String, ToString};
use alloc::vec::Vec;

use bitcoin::bech32::{self, FromBase32, ToBase32, Variant};
use bitcoin::secp256k1::XOnlyPublicKey;

use crate::nips::nip19::{
    Error as Bech32Error, FromBech32, ToBech32, AUTHOR, KIND,
    PREFIX_BECH32_PARAMETERIZED_REPLACEABLE_EVENT, RELAY, SPECIAL,
};
use crate::{Kind, Tag, UncheckedUrl};

/// Parameterized Replaceable Event
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ParameterizedReplaceableEvent {
    /// Kind
    pub kind: Kind,
    /// Public Key
    pub pubkey: XOnlyPublicKey,
    /// `d` tag identifier
    pub identifier: String,
    /// Relays
    pub relays: Vec<String>,
}

impl From<ParameterizedReplaceableEvent> for Tag {
    fn from(value: ParameterizedReplaceableEvent) -> Self {
        Self::A {
            kind: value.kind,
            public_key: value.pubkey,
            identifier: value.identifier,
            relay_url: value.relays.first().map(UncheckedUrl::from),
        }
    }
}

impl FromBech32 for ParameterizedReplaceableEvent {
    type Err = Bech32Error;
    fn from_bech32<S>(s: S) -> Result<Self, Self::Err>
    where
        S: Into<String>,
    {
        let (hrp, data, checksum) = bech32::decode(&s.into())?;

        if hrp != PREFIX_BECH32_PARAMETERIZED_REPLACEABLE_EVENT || checksum != Variant::Bech32 {
            return Err(Bech32Error::WrongPrefixOrVariant);
        }

        let mut data: Vec<u8> = Vec::from_base32(&data)?;

        let mut identifier: Option<String> = None;
        let mut pubkey: Option<XOnlyPublicKey> = None;
        let mut kind: Option<Kind> = None;
        let mut relays: Vec<String> = Vec::new();

        while !data.is_empty() {
            let t = data.first().ok_or(Bech32Error::TLV)?;
            let l = data.get(1).ok_or(Bech32Error::TLV)?;
            let l = *l as usize;

            let bytes = data.get(2..l + 2).ok_or(Bech32Error::TLV)?;

            match *t {
                SPECIAL => {
                    if identifier.is_none() {
                        identifier = Some(String::from_utf8(bytes.to_vec())?);
                    }
                }
                RELAY => {
                    relays.push(String::from_utf8(bytes.to_vec())?);
                }
                AUTHOR => {
                    if pubkey.is_none() {
                        pubkey = Some(XOnlyPublicKey::from_slice(bytes)?);
                    }
                }
                KIND => {
                    if kind.is_none() {
                        let k: u64 = u32::from_be_bytes(
                            bytes.try_into().map_err(|_| Bech32Error::TryFromSlice)?,
                        ) as u64;
                        kind = Some(Kind::from(k));
                    }
                }
                _ => (),
            };

            data.drain(..l + 2);
        }

        Ok(Self {
            kind: kind.ok_or_else(|| Bech32Error::FieldMissing("kind".to_string()))?,
            pubkey: pubkey.ok_or_else(|| Bech32Error::FieldMissing("pubkey".to_string()))?,
            identifier: identifier
                .ok_or_else(|| Bech32Error::FieldMissing("identifier".to_string()))?,
            relays,
        })
    }
}

impl ToBech32 for ParameterizedReplaceableEvent {
    type Err = Bech32Error;

    fn to_bech32(&self) -> Result<String, Self::Err> {
        let mut bytes: Vec<u8> = Vec::new();

        // Identifier
        bytes.extend([SPECIAL, self.identifier.len() as u8]);
        bytes.extend(self.identifier.as_bytes());

        for relay in self.relays.iter() {
            bytes.extend([RELAY, relay.len() as u8]);
            bytes.extend(relay.as_bytes());
        }

        // Author
        bytes.extend([AUTHOR, 32]);
        bytes.extend(self.pubkey.serialize());

        // Kind
        bytes.extend([KIND, 4]);
        bytes.extend(self.kind.as_u32().to_be_bytes());

        let data = bytes.to_base32();
        Ok(bech32::encode(
            PREFIX_BECH32_PARAMETERIZED_REPLACEABLE_EVENT,
            data,
            Variant::Bech32,
        )?)
    }
}
