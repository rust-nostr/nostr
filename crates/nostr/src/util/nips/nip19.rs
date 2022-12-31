// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use bitcoin::bech32::{self, FromBase32, ToBase32, Variant};
use bitcoin::secp256k1::{SecretKey, XOnlyPublicKey};

use crate::Sha256Hash;

const PREFIX_BECH32_SECRET_KEY: &str = "nsec";
const PREFIX_BECH32_PUBLIC_KEY: &str = "npub";
const PREFIX_BECH32_NOTE_ID: &str = "note";
const PREFIX_BECH32_PROFILE: &str = "nprofile";
const PREFIX_BECH32_EVENT: &str = "nevent";

#[derive(Debug, Eq, PartialEq, thiserror::Error)]
pub enum Error {
    /// Bech32 encoding error.
    #[error("Bech32 key encoding error: {0}")]
    Bech32(#[from] bech32::Error),
    #[error("Invalid bech32 secret key")]
    Bech32SkParseError,
    #[error("Invalid bech32 public key")]
    Bech32PkParseError,
    /// Secp256k1 error
    #[error("secp256k1 error: {0}")]
    Secp256k1(#[from] bitcoin::secp256k1::Error),
}

pub trait FromBech32: Sized {
    fn from_bech32<S>(s: S) -> Result<Self, Error>
    where
        S: Into<String>;
}

impl FromBech32 for SecretKey {
    fn from_bech32<S>(secret_key: S) -> Result<Self, Error>
    where
        S: Into<String>,
    {
        let (hrp, data, checksum) =
            bech32::decode(&secret_key.into()).map_err(|_| Error::Bech32SkParseError)?;

        if hrp != PREFIX_BECH32_SECRET_KEY || checksum != Variant::Bech32 {
            return Err(Error::Bech32SkParseError);
        }

        let data = Vec::<u8>::from_base32(&data).map_err(|_| Error::Bech32SkParseError)?;
        SecretKey::from_slice(data.as_slice()).map_err(|_| Error::Bech32SkParseError)
    }
}

impl FromBech32 for XOnlyPublicKey {
    fn from_bech32<S>(public_key: S) -> Result<Self, Error>
    where
        S: Into<String>,
    {
        let (hrp, data, checksum) =
            bech32::decode(&public_key.into()).map_err(|_| Error::Bech32PkParseError)?;

        if hrp != PREFIX_BECH32_PUBLIC_KEY || checksum != Variant::Bech32 {
            return Err(Error::Bech32PkParseError);
        }

        let data = Vec::<u8>::from_base32(&data).map_err(|_| Error::Bech32PkParseError)?;
        Ok(XOnlyPublicKey::from_slice(data.as_slice())?)
    }
}

pub trait ToBech32 {
    type Err;
    fn to_bech32(&self) -> Result<String, Self::Err>;
}

impl ToBech32 for XOnlyPublicKey {
    type Err = Error;

    fn to_bech32(&self) -> Result<String, Self::Err> {
        let data = self.serialize().to_base32();
        Ok(bech32::encode(
            PREFIX_BECH32_PUBLIC_KEY,
            data,
            Variant::Bech32,
        )?)
    }
}

impl ToBech32 for SecretKey {
    type Err = Error;

    fn to_bech32(&self) -> Result<String, Self::Err> {
        let data = self.secret_bytes().to_base32();
        Ok(bech32::encode(
            PREFIX_BECH32_SECRET_KEY,
            data,
            Variant::Bech32,
        )?)
    }
}

// Note ID
impl ToBech32 for Sha256Hash {
    type Err = Error;

    fn to_bech32(&self) -> Result<String, Self::Err> {
        let data = self.to_base32();
        Ok(bech32::encode(
            PREFIX_BECH32_NOTE_ID,
            data,
            Variant::Bech32,
        )?)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct Profile {
    public_key: XOnlyPublicKey,
    relays: Vec<String>,
}

impl Profile {
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

impl ToBech32 for Profile {
    type Err = Error;

    fn to_bech32(&self) -> Result<String, Self::Err> {
        let mut bytes: Vec<u8> = vec![0, 32];
        bytes.extend(self.public_key.serialize());

        for relay in self.relays.iter() {
            bytes.extend([1, relay.len() as u8]);
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

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct Event {
    event_id: Sha256Hash,
    relays: Vec<String>,
}

impl Event {
    pub fn new<S>(event_id: Sha256Hash, relays: Vec<S>) -> Self
    where
        S: Into<String>,
    {
        Self {
            event_id,
            relays: relays.into_iter().map(|u| u.into()).collect(),
        }
    }
}

impl ToBech32 for Event {
    type Err = Error;

    fn to_bech32(&self) -> Result<String, Self::Err> {
        let mut bytes: Vec<u8> = vec![0, 32];
        bytes.extend(self.event_id.iter());

        for relay in self.relays.iter() {
            bytes.extend([1, relay.len() as u8]);
            bytes.extend(relay.as_bytes());
        }

        let data = bytes.to_base32();
        Ok(bech32::encode(PREFIX_BECH32_EVENT, data, Variant::Bech32)?)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use crate::Result;

    #[test]
    fn to_bech32_public_key() -> Result<()> {
        let public_key = XOnlyPublicKey::from_str(
            "aa4fc8665f5696e33db7e1a572e3b0f5b3d615837b0f362dcb1c8068b098c7b4",
        )?;
        assert_eq!(
            "npub14f8usejl26twx0dhuxjh9cas7keav9vr0v8nvtwtrjqx3vycc76qqh9nsy".to_string(),
            public_key.to_bech32()?
        );
        Ok(())
    }

    #[test]
    fn to_bech32_secret_key() -> Result<()> {
        let secret_key = SecretKey::from_str(
            "9571a568a42b9e05646a349c783159b906b498119390df9a5a02667155128028",
        )?;
        assert_eq!(
            "nsec1j4c6269y9w0q2er2xjw8sv2ehyrtfxq3jwgdlxj6qfn8z4gjsq5qfvfk99".to_string(),
            secret_key.to_bech32()?
        );
        Ok(())
    }

    #[test]
    fn to_bech32_note() -> Result<()> {
        let event_id = Sha256Hash::from_str(
            "d94a3f4dd87b9a3b0bed183b32e916fa29c8020107845d1752d72697fe5309a5",
        )?;
        assert_eq!(
            "note1m99r7nwc0wdrkzldrqan96gklg5usqspq7z9696j6unf0ljnpxjspqfw99".to_string(),
            event_id.to_bech32()?
        );
        Ok(())
    }

    #[test]
    fn to_bech32_profile() -> Result<()> {
        let profile = Profile::new(
            XOnlyPublicKey::from_str(
                "3bf0c63fcb93463407af97a5e5ee64fa883d107ef9e558472c4eb9aaaefa459d",
            )?,
            vec![
                String::from("wss://r.x.com"),
                String::from("wss://djbas.sadkb.com"),
            ],
        );
        assert_eq!("nprofile1qqsrhuxx8l9ex335q7he0f09aej04zpazpl0ne2cgukyawd24mayt8gpp4mhxue69uhhytnc9e3k7mgpz4mhxue69uhkg6nzv9ejuumpv34kytnrdaksjlyr9p".to_string(), profile.to_bech32()?);
        Ok(())
    }
}
