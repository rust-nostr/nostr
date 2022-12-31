// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use bitcoin::bech32::{self, FromBase32, ToBase32, Variant};
use bitcoin::secp256k1::{SecretKey, XOnlyPublicKey};

use crate::{Sha256Hash, Keys};

const PREFIX_BECH32_SECRET_KEY: &str = "nsec";
const PREFIX_BECH32_PUBLIC_KEY: &str = "npub";
const PREFIX_BECH32_NOTE_ID: &str = "note";

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
    fn from_bech32<S>(secret_key: S) -> Result<Self, Error>
    where
        S: Into<String>;
    fn from_bech32_public_key<S>(public_key: S) -> Result<Self, Error>
    where
        S: Into<String>;
}

impl FromBech32 for Keys {
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
        let secret_key =
            SecretKey::from_slice(data.as_slice()).map_err(|_| Error::Bech32SkParseError)?;
        Ok(Self::new(secret_key))
    }

    fn from_bech32_public_key<S>(public_key: S) -> Result<Self, Error>
    where
        S: Into<String>,
    {
        let (hrp, data, checksum) =
            bech32::decode(&public_key.into()).map_err(|_| Error::Bech32PkParseError)?;

        if hrp != PREFIX_BECH32_PUBLIC_KEY || checksum != Variant::Bech32 {
            return Err(Error::Bech32PkParseError);
        }

        let data = Vec::<u8>::from_base32(&data).map_err(|_| Error::Bech32PkParseError)?;
        let public_key = XOnlyPublicKey::from_slice(data.as_slice())?;
        Ok(Self::from_public_key(public_key))
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

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use crate::Result;

    #[test]
    fn to_bech32_public_key() -> Result<()> {
        let bech32_pubkey_str: &str =
            "npub14f8usejl26twx0dhuxjh9cas7keav9vr0v8nvtwtrjqx3vycc76qqh9nsy";
        let keys = Keys::from_bech32_public_key(bech32_pubkey_str)?;
        let public_key: XOnlyPublicKey = keys.public_key();

        assert_eq!(bech32_pubkey_str.to_string(), public_key.to_bech32()?);

        Ok(())
    }

    #[test]
    fn to_bech32_secret_key() -> Result<()> {
        let bech32_secret_key_str: &str =
            "nsec1j4c6269y9w0q2er2xjw8sv2ehyrtfxq3jwgdlxj6qfn8z4gjsq5qfvfk99";
        let keys = Keys::from_bech32(bech32_secret_key_str)?;
        let secret_key: SecretKey = keys.secret_key()?;

        assert_eq!(bech32_secret_key_str.to_string(), secret_key.to_bech32()?);

        Ok(())
    }

    #[test]
    fn to_bech32_note() -> Result<()> {
        let event_id = Sha256Hash::from_str("d94a3f4dd87b9a3b0bed183b32e916fa29c8020107845d1752d72697fe5309a5")?;
        assert_eq!("note1m99r7nwc0wdrkzldrqan96gklg5usqspq7z9696j6unf0ljnpxjspqfw99".to_string(), event_id.to_bech32()?);
        Ok(())
    }
}
