// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP44: Versioned Encryption
//!
//! <https://github.com/nostr-protocol/nips/blob/master/44.md>

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use base64::engine::{general_purpose, Engine};
#[cfg(feature = "std")]
use bitcoin::secp256k1::rand::rngs::OsRng;
use bitcoin::secp256k1::rand::RngCore;

pub mod v2;

use self::v2::ConversationKey;
use crate::{PublicKey, SecretKey};

/// Error
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// NIP44 V2 error
    V2(v2::ErrorV2),
    /// Error while decoding from base64
    Base64Decode(base64::DecodeError),
    /// Invalid length
    InvalidLength,
    /// Error while encoding to UTF-8
    Utf8Encode,
    /// Unknown version
    UnknownVersion(u8),
    /// Version not found in payload
    VersionNotFound,
    /// Not found in payload
    NotFound(String),
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::V2(e) => write!(f, "{e}"),
            Self::Base64Decode(e) => write!(f, "Error while decoding from base64: {e}"),
            Self::InvalidLength => write!(f, "Invalid length"),
            Self::Utf8Encode => write!(f, "Error while encoding to UTF-8"),
            Self::UnknownVersion(v) => write!(f, "unknown version: {v}"),
            Self::VersionNotFound => write!(f, "Version not found in payload"),
            Self::NotFound(value) => write!(f, "{value} not found in payload"),
        }
    }
}

impl From<v2::ErrorV2> for Error {
    fn from(e: v2::ErrorV2) -> Self {
        Self::V2(e)
    }
}

impl From<base64::DecodeError> for Error {
    fn from(e: base64::DecodeError) -> Self {
        Self::Base64Decode(e)
    }
}

/// Payload version
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Version {
    /// V2 - Secp256k1 ECDH, HKDF, padding, ChaCha20, HMAC-SHA256 and base64
    #[default]
    V2 = 0x02,
}

impl Version {
    /// Get [`Version`] as `u8`
    #[inline]
    pub fn as_u8(&self) -> u8 {
        *self as u8
    }
}

impl TryFrom<u8> for Version {
    type Error = Error;

    fn try_from(version: u8) -> Result<Self, Self::Error> {
        match version {
            0x02 => Ok(Self::V2),
            v => Err(Error::UnknownVersion(v)),
        }
    }
}

/// Encrypt
#[inline]
#[cfg(feature = "std")]
pub fn encrypt<T>(
    secret_key: &SecretKey,
    public_key: &PublicKey,
    content: T,
    version: Version,
) -> Result<String, Error>
where
    T: AsRef<[u8]>,
{
    encrypt_with_rng(&mut OsRng, secret_key, public_key, content, version)
}

/// Encrypt
pub fn encrypt_with_rng<R, T>(
    rng: &mut R,
    secret_key: &SecretKey,
    public_key: &PublicKey,
    content: T,
    version: Version,
) -> Result<String, Error>
where
    R: RngCore,
    T: AsRef<[u8]>,
{
    match version {
        Version::V2 => {
            let conversation_key: ConversationKey = ConversationKey::derive(secret_key, public_key);
            let payload: Vec<u8> = v2::encrypt_to_bytes_with_rng(rng, &conversation_key, content)?;
            Ok(general_purpose::STANDARD.encode(payload))
        }
    }
}

/// Decrypt
#[inline]
pub fn decrypt<T>(
    secret_key: &SecretKey,
    public_key: &PublicKey,
    payload: T,
) -> Result<String, Error>
where
    T: AsRef<[u8]>,
{
    let bytes: Vec<u8> = decrypt_to_bytes(secret_key, public_key, payload)?;
    String::from_utf8(bytes.to_vec()).map_err(|_| Error::Utf8Encode)
}

/// Decrypt **without** converting bytes to UTF-8 string
pub fn decrypt_to_bytes<T>(
    secret_key: &SecretKey,
    public_key: &PublicKey,
    payload: T,
) -> Result<Vec<u8>, Error>
where
    T: AsRef<[u8]>,
{
    // Decode base64 payload
    let payload: Vec<u8> = general_purpose::STANDARD.decode(payload)?;

    // Get version byte
    let version: u8 = *payload.first().ok_or(Error::VersionNotFound)?;

    match Version::try_from(version)? {
        Version::V2 => {
            let conversation_key: ConversationKey = ConversationKey::derive(secret_key, public_key);
            v2::decrypt_to_bytes(&conversation_key, &payload)
        }
    }
}

#[cfg(test)]
#[cfg(feature = "std")]
mod tests {
    use core::str::FromStr;

    use super::*;
    use crate::Keys;

    #[test]
    fn test_nip44_encryption_decryption() {
        // Alice keys
        let alice_sk =
            SecretKey::from_str("5c0c523f52a5b6fad39ed2403092df8cebc36318b39383bca6c00808626fab3a")
                .unwrap();
        let alice_keys = Keys::new(alice_sk);
        let alice_pk = alice_keys.public_key();

        // Bob keys
        let bob_sk =
            SecretKey::from_str("4b22aa260e4acb7021e32f38a6cdf4b673c6a277755bfce287e370c924dc936d")
                .unwrap();
        let bob_keys = Keys::new(bob_sk);
        let bob_pk = bob_keys.public_key();

        let content = String::from("hello");
        let encrypted_content =
            encrypt(alice_keys.secret_key(), &bob_pk, &content, Version::V2).unwrap();
        assert_eq!(
            decrypt(bob_keys.secret_key(), &alice_pk, encrypted_content).unwrap(),
            content
        );
    }
}
