// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Encrypted Secret Key
//!
//! <https://github.com/nostr-protocol/nips/blob/master/49.md>

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::array::TryFromSliceError;
use core::fmt;

#[cfg(feature = "std")]
use bitcoin::secp256k1::rand::rngs::OsRng;
use bitcoin::secp256k1::rand::{CryptoRng, RngCore};
use chacha20poly1305::aead::{Aead, AeadCore, KeyInit, Payload};
use chacha20poly1305::XChaCha20Poly1305;
use scrypt::errors::{InvalidOutputLen, InvalidParams};
use scrypt::Params as ScryptParams;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use unicode_normalization::UnicodeNormalization;

use super::nip19::{FromBech32, ToBech32};
use crate::{key, SecretKey};

const SALT_SIZE: usize = 16;
const NONCE_SIZE: usize = 24;
const CIPHERTEXT_SIZE: usize = 48;
const TOTAL_SIZE: usize = 1 + 1 + SALT_SIZE + NONCE_SIZE + 1 + CIPHERTEXT_SIZE; // 91
const KEY_SIZE: usize = 32;

/// NIP49 error
#[derive(Debug, Eq, PartialEq)]
pub enum Error {
    /// Try from slice
    TryFromSlice(String),
    /// ChaCha20Poly1305 error
    ChaCha20Poly1305(chacha20poly1305::Error),
    /// Invalid scrypt params
    InvalidScryptParams(InvalidParams),
    /// Invalid scrypt output len
    InvalidScryptOutputLen(InvalidOutputLen),
    /// Keys error
    Keys(key::Error),
    /// Invalid len
    InvalidLength {
        /// Expected bytes len
        expected: usize,
        /// Found bytes len
        found: usize,
    },
    /// Unsupported version
    UnsupportedVersion(u8),
    /// Unknown version
    UnknownVersion(u8),
    /// Unknown Key Security
    UnknownKeySecurity(u8),
    /// Version not found
    VersionNotFound,
    /// Log2 round not found
    Log2RoundNotFound,
    /// Salt not found
    SaltNotFound,
    /// Nonce not found
    NonceNotFound,
    /// Key security not found
    KeySecurityNotFound,
    /// Cipthertext not found
    CipherTextNotFound,
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TryFromSlice(e) => write!(f, "{e}"),
            Self::ChaCha20Poly1305(e) => write!(f, "ChaCha20Poly1305: {e}"),
            Self::InvalidScryptParams(e) => write!(f, "Invalid scrypt params: {e}"),
            Self::InvalidScryptOutputLen(e) => write!(f, "Invalid scrypt output len: {e}"),
            Self::Keys(e) => write!(f, "Keys: {e}"),
            Self::InvalidLength { expected, found } => write!(
                f,
                "Invalid encrypted secret key bytes len: expected={expected}, found={found}"
            ),
            Self::UnsupportedVersion(v) => write!(
                f,
                "Unsupported encrypted secret key version: {v} (deprecated)"
            ),
            Self::UnknownVersion(v) => write!(f, "Unknown encrypted secret key version: {v}"),
            Self::UnknownKeySecurity(v) => write!(f, "Unknown encrypted secret key security: {v}"),
            Self::VersionNotFound => write!(f, "Encrypted secret key version not found"),
            Self::Log2RoundNotFound => write!(f, "Encrypted secret key `log N` not found"),
            Self::SaltNotFound => write!(f, "Encrypted secret key salt not found"),
            Self::NonceNotFound => write!(f, "Encrypted secret key nonce not found"),
            Self::KeySecurityNotFound => write!(f, "Encrypted secret key security not found"),
            Self::CipherTextNotFound => write!(f, "Encrypted secret key ciphertext not found"),
        }
    }
}

impl From<TryFromSliceError> for Error {
    fn from(e: TryFromSliceError) -> Self {
        Self::TryFromSlice(e.to_string())
    }
}

impl From<chacha20poly1305::Error> for Error {
    fn from(e: chacha20poly1305::Error) -> Self {
        Self::ChaCha20Poly1305(e)
    }
}

impl From<InvalidParams> for Error {
    fn from(e: InvalidParams) -> Self {
        Self::InvalidScryptParams(e)
    }
}

impl From<InvalidOutputLen> for Error {
    fn from(e: InvalidOutputLen) -> Self {
        Self::InvalidScryptOutputLen(e)
    }
}

impl From<key::Error> for Error {
    fn from(e: key::Error) -> Self {
        Self::Keys(e)
    }
}

/// Encrypted Secret Key version (NIP49)
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Version {
    /// V2
    #[default]
    V2 = 0x02,
}

impl TryFrom<u8> for Version {
    type Error = Error;

    fn try_from(version: u8) -> Result<Self, Self::Error> {
        match version {
            0x01 => Err(Error::UnsupportedVersion(version)),
            0x02 => Ok(Self::V2),
            v => Err(Error::UnknownVersion(v)),
        }
    }
}

/// Key security
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum KeySecurity {
    /// The key has been known to have been handled insecurely (stored unencrypted, cut and paste unencrypted, etc)
    Weak = 0x00,
    /// The key has NOT been known to have been handled insecurely (stored encrypted, cut and paste encrypted, etc)
    Medium = 0x01,
    /// The client does not track this data
    #[default]
    Unknown = 0x02,
}

impl TryFrom<u8> for KeySecurity {
    type Error = Error;

    fn try_from(key_security: u8) -> Result<Self, Self::Error> {
        match key_security {
            0x00 => Ok(Self::Weak),
            0x01 => Ok(Self::Medium),
            0x02 => Ok(Self::Unknown),
            v => Err(Error::UnknownKeySecurity(v)),
        }
    }
}

/// Encrypted Secret Key
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EncryptedSecretKey {
    version: Version,
    log_n: u8,
    salt: [u8; SALT_SIZE],
    nonce: [u8; NONCE_SIZE],
    key_security: KeySecurity,
    ciphertext: [u8; CIPHERTEXT_SIZE],
}

impl EncryptedSecretKey {
    /// Encrypt [SecretKey]
    #[cfg(feature = "std")]
    pub fn new<S>(
        secret_key: &SecretKey,
        password: S,
        log_n: u8,
        key_security: KeySecurity,
    ) -> Result<Self, Error>
    where
        S: AsRef<str>,
    {
        Self::new_with_rng(&mut OsRng, secret_key, password, log_n, key_security)
    }

    /// Encrypt [SecretKey]
    pub fn new_with_rng<R, S>(
        rng: &mut R,
        secret_key: &SecretKey,
        password: S,
        log_n: u8,
        key_security: KeySecurity,
    ) -> Result<Self, Error>
    where
        R: RngCore + CryptoRng,
        S: AsRef<str>,
    {
        // Generate salt
        let salt: [u8; SALT_SIZE] = {
            let mut salt: [u8; SALT_SIZE] = [0u8; SALT_SIZE];
            rng.fill_bytes(&mut salt);
            salt
        };

        // Generate nonce
        let nonce = XChaCha20Poly1305::generate_nonce(rng);

        // Derive key
        let key: [u8; KEY_SIZE] = derive_key(password, &salt, log_n)?;

        // Compose cipher
        let cipher = XChaCha20Poly1305::new(&key.into());

        // Compose payload
        let payload = Payload {
            msg: &secret_key.to_secret_bytes(),
            aad: &[key_security as u8],
        };

        // Encrypt
        let ciphertext: Vec<u8> = cipher.encrypt(&nonce, payload)?;
        let ciphertext: [u8; CIPHERTEXT_SIZE] = ciphertext.as_slice().try_into()?;

        Ok(Self {
            version: Version::default(),
            log_n,
            salt,
            nonce: nonce.into(),
            key_security,
            ciphertext,
        })
    }

    /// Parse encrypted secret key from bytes
    pub fn from_slice(slice: &[u8]) -> Result<Self, Error> {
        if slice.len() != TOTAL_SIZE {
            return Err(Error::InvalidLength {
                expected: TOTAL_SIZE,
                found: slice.len(),
            });
        }

        // Version
        let version: u8 = slice.first().copied().ok_or(Error::VersionNotFound)?;
        let version: Version = Version::try_from(version)?;

        // Log 2 rounds
        let log_n: u8 = slice.get(1).copied().ok_or(Error::Log2RoundNotFound)?;

        // Salt
        let salt: &[u8] = slice.get(2..2 + SALT_SIZE).ok_or(Error::SaltNotFound)?;
        let salt: [u8; SALT_SIZE] = salt.try_into()?;

        // Nonce
        let nonce: &[u8] = slice
            .get(2 + SALT_SIZE..2 + SALT_SIZE + NONCE_SIZE)
            .ok_or(Error::NonceNotFound)?;
        let nonce: [u8; NONCE_SIZE] = nonce.try_into()?;

        // Key security
        let key_security: u8 = slice
            .get(2 + SALT_SIZE + NONCE_SIZE)
            .copied()
            .ok_or(Error::KeySecurityNotFound)?;
        let key_security: KeySecurity = KeySecurity::try_from(key_security)?;

        // Ciphertext
        let ciphertext: &[u8] = slice
            .get(2 + SALT_SIZE + NONCE_SIZE + 1..)
            .ok_or(Error::CipherTextNotFound)?;
        let ciphertext: [u8; CIPHERTEXT_SIZE] = ciphertext.try_into()?;

        Ok(Self {
            version,
            log_n,
            salt,
            nonce,
            key_security,
            ciphertext,
        })
    }

    /// Get encrypted secret key as bytes
    pub fn as_vec(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::with_capacity(TOTAL_SIZE);
        bytes.push(self.version as u8);
        bytes.push(self.log_n);
        bytes.extend_from_slice(&self.salt);
        bytes.extend_from_slice(&self.nonce);
        bytes.push(self.key_security as u8);
        bytes.extend_from_slice(&self.ciphertext);
        bytes
    }

    /// Get encrypted secret key version
    pub fn version(&self) -> Version {
        self.version
    }

    /// Get encrypted secret key security
    pub fn key_security(&self) -> KeySecurity {
        self.key_security
    }

    /// Decrypt secret key
    pub fn to_secret_key<S>(self, password: S) -> Result<SecretKey, Error>
    where
        S: AsRef<str>,
    {
        // Derive key
        let key: [u8; KEY_SIZE] = derive_key(password, &self.salt, self.log_n)?;

        // Compose cipher
        let cipher = XChaCha20Poly1305::new(&key.into());

        // Compose payload
        let payload = Payload {
            msg: &self.ciphertext,
            aad: &[self.key_security as u8],
        };

        // Decrypt
        let bytes: Vec<u8> = cipher.decrypt(&self.nonce.into(), payload)?;

        Ok(SecretKey::from_slice(&bytes)?)
    }
}

impl Serialize for EncryptedSecretKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let cryptsec: String = self.to_bech32();
        serializer.serialize_str(&cryptsec)
    }
}

impl<'de> Deserialize<'de> for EncryptedSecretKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let cryptsec: String = String::deserialize(deserializer)?;
        Self::from_bech32(cryptsec).map_err(serde::de::Error::custom)
    }
}

fn derive_key<S>(password: S, salt: &[u8; SALT_SIZE], log_n: u8) -> Result<[u8; KEY_SIZE], Error>
where
    S: AsRef<str>,
{
    // Unicode Normalization
    let password: &str = password.as_ref();
    let password: String = password.nfkc().collect();

    // Compose params
    let params: ScryptParams = ScryptParams::new(log_n, 8, 1, KEY_SIZE)?;

    // Derive key
    let mut key: [u8; KEY_SIZE] = [0u8; KEY_SIZE];
    scrypt::scrypt(password.as_bytes(), salt, &params, &mut key)?;
    Ok(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    const CRYPTSEC: &str = "ncryptsec1qgg9947rlpvqu76pj5ecreduf9jxhselq2nae2kghhvd5g7dgjtcxfqtd67p9m0w57lspw8gsq6yphnm8623nsl8xn9j4jdzz84zm3frztj3z7s35vpzmqf6ksu8r89qk5z2zxfmu5gv8th8wclt0h4p";
    const SECRET_KEY: &str = "3501454135014541350145413501453fefb02227e449e57cf4d3a3ce05378683";

    #[test]
    fn test_encrypted_secret_key_decryption() {
        let encrypted_secret_key = EncryptedSecretKey::from_bech32(CRYPTSEC).unwrap();
        let secret_key: SecretKey = encrypted_secret_key.to_secret_key("nostr").unwrap();
        assert_eq!(secret_key.to_secret_hex(), SECRET_KEY)
    }

    #[test]
    fn test_encrypted_secret_key_serialization() {
        let encrypted_secret_key = EncryptedSecretKey::from_bech32(CRYPTSEC).unwrap();
        assert_eq!(encrypted_secret_key.to_bech32(), CRYPTSEC)
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_encrypted_secret_key_encryption_decryption() {
        let original_secret_key = SecretKey::from_hex(SECRET_KEY).unwrap();
        let encrypted_secret_key =
            EncryptedSecretKey::new(&original_secret_key, "test", 16, KeySecurity::Medium).unwrap();
        let secret_key: SecretKey = encrypted_secret_key.to_secret_key("test").unwrap();
        assert_eq!(original_secret_key, secret_key);
        assert_eq!(encrypted_secret_key.version(), Version::default());
        assert_eq!(encrypted_secret_key.key_security(), KeySecurity::Medium);
    }
}
