// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Profile

use alloc::string::{String, ToString};
use alloc::vec::Vec;

use bitcoin::bech32::{self, FromBase32, ToBase32, Variant};
use bitcoin::secp256k1::XOnlyPublicKey;

use crate::nips::nip19::{Error, FromBech32, ToBech32, PREFIX_BECH32_PROFILE, RELAY, SPECIAL};

/// Profile
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Profile {
    /// Public key
    pub public_key: XOnlyPublicKey,
    /// Relays
    pub relays: Vec<String>,
}

impl Profile {
    /// New [`Profile`]
    pub fn new<S>(public_key: XOnlyPublicKey, relays: Vec<S>) -> Self
    where
        S: Into<String>,
    {
        Self {
            public_key,
            relays: relays.into_iter().map(|u| u.into()).collect(),
        }
    }
}

impl FromBech32 for Profile {
    type Err = Error;
    fn from_bech32<S>(s: S) -> Result<Self, Self::Err>
    where
        S: Into<String>,
    {
        let (hrp, data, checksum) = bech32::decode(&s.into())?;

        if hrp != PREFIX_BECH32_PROFILE || checksum != Variant::Bech32 {
            return Err(Error::WrongPrefixOrVariant);
        }

        let mut data: Vec<u8> = Vec::from_base32(&data)?;

        let mut pubkey: Option<XOnlyPublicKey> = None;
        let mut relays: Vec<String> = Vec::new();

        while !data.is_empty() {
            let t = data.first().ok_or(Error::TLV)?;
            let l = data.get(1).ok_or(Error::TLV)?;
            let l = *l as usize;

            let bytes = data.get(2..l + 2).ok_or(Error::TLV)?;

            match *t {
                SPECIAL => {
                    if pubkey.is_none() {
                        pubkey = Some(XOnlyPublicKey::from_slice(bytes)?);
                    }
                }
                RELAY => {
                    relays.push(String::from_utf8(bytes.to_vec())?);
                }
                _ => (),
            };

            data.drain(..l + 2);
        }

        Ok(Self {
            public_key: pubkey.ok_or_else(|| Error::FieldMissing("pubkey".to_string()))?,
            relays,
        })
    }
}

impl ToBech32 for Profile {
    type Err = Error;

    fn to_bech32(&self) -> Result<String, Self::Err> {
        let mut bytes: Vec<u8> = vec![SPECIAL, 32];
        bytes.extend(self.public_key.serialize());

        for relay in self.relays.iter() {
            bytes.extend([RELAY, relay.len() as u8]);
            bytes.extend(relay.as_bytes());
        }

        let data = bytes.to_base32();
        Ok(bech32::encode(
            PREFIX_BECH32_PROFILE,
            data,
            Variant::Bech32,
        )?)
    }
}

#[cfg(test)]
mod tests {
    use core::str::FromStr;

    use super::*;

    #[test]
    fn to_bech32_profile() {
        let profile = Profile::new(
            XOnlyPublicKey::from_str(
                "3bf0c63fcb93463407af97a5e5ee64fa883d107ef9e558472c4eb9aaaefa459d",
            )
            .unwrap(),
            vec![
                String::from("wss://r.x.com"),
                String::from("wss://djbas.sadkb.com"),
            ],
        );
        assert_eq!("nprofile1qqsrhuxx8l9ex335q7he0f09aej04zpazpl0ne2cgukyawd24mayt8gpp4mhxue69uhhytnc9e3k7mgpz4mhxue69uhkg6nzv9ejuumpv34kytnrdaksjlyr9p".to_string(), profile.to_bech32().unwrap());
    }

    #[test]
    fn from_bech32_profile() {
        let bech32_profile = "nprofile1qqsrhuxx8l9ex335q7he0f09aej04zpazpl0ne2cgukyawd24mayt8gpp4mhxue69uhhytnc9e3k7mgpz4mhxue69uhkg6nzv9ejuumpv34kytnrdaksjlyr9p";
        let profile = Profile::from_bech32(bech32_profile).unwrap();
        assert_eq!(
            "3bf0c63fcb93463407af97a5e5ee64fa883d107ef9e558472c4eb9aaaefa459d".to_string(),
            profile.public_key.to_string()
        );
        assert_eq!(
            vec![
                "wss://r.x.com".to_string(),
                "wss://djbas.sadkb.com".to_string()
            ],
            profile.relays
        );
    }
}
