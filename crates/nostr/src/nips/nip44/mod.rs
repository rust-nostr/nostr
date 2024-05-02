// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP44
//!
//! <https://github.com/nostr-protocol/nips/blob/master/44.md>

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use base64::engine::{general_purpose, Engine};
use bitcoin::hashes::sha256::Hash as Sha256Hash;
use bitcoin::hashes::Hash;
#[cfg(feature = "std")]
use bitcoin::secp256k1::rand::rngs::OsRng;
use bitcoin::secp256k1::rand::RngCore;
use chacha20::cipher::{KeyIvInit, StreamCipher};
use chacha20::XChaCha20;

pub mod v2;

use self::v2::ConversationKey;
use crate::{secp256k1, util, PublicKey, SecretKey};

/// Error
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// NIP44 V2 error
    V2(v2::ErrorV2),
    /// Error while decoding from base64
    Base64Decode(base64::DecodeError),
    /// Secp256k1 error
    Secp256k1(secp256k1::Error),
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
            Self::Secp256k1(e) => write!(f, "Secp256k1: {e}"),
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

impl From<secp256k1::Error> for Error {
    fn from(e: secp256k1::Error) -> Self {
        Self::Secp256k1(e)
    }
}

/// Payload version
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Version {
    /// Reserved
    // Reserved = 0x00,
    /// V1 (deprecated)
    #[deprecated]
    V1 = 0x01,
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
            #[allow(deprecated)]
            0x01 => Ok(Self::V1),
            0x02 => Ok(Self::V2),
            v => Err(Error::UnknownVersion(v)),
        }
    }
}

/// Encrypt - EXPERIMENTAL
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

/// Encrypt - EXPERIMENTAL
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
        #[allow(deprecated)]
        Version::V1 => {
            // Compose key
            let shared_key: [u8; 32] = util::generate_shared_key(secret_key, public_key)?;
            let key: Sha256Hash = Sha256Hash::hash(&shared_key);

            // Generate 192-bit nonce
            let mut nonce: [u8; 24] = [0u8; 24];
            rng.fill_bytes(&mut nonce);

            // Compose cipher
            let mut cipher = XChaCha20::new(key.as_byte_array().into(), &nonce.into());

            // Encrypt
            let mut buffer: Vec<u8> = content.as_ref().to_vec();
            cipher.apply_keystream(&mut buffer);

            // Compose payload
            let mut payload: Vec<u8> = vec![version.as_u8()];
            payload.extend_from_slice(nonce.as_slice());
            payload.extend(buffer);

            // Encode payload to base64
            Ok(general_purpose::STANDARD.encode(payload))
        }
        Version::V2 => {
            let conversation_key: ConversationKey =
                ConversationKey::derive(secret_key, public_key)?;
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
        #[allow(deprecated)]
        Version::V1 => {
            // Get data from payload
            let nonce: &[u8] = payload
                .get(1..25)
                .ok_or_else(|| Error::NotFound(String::from("nonce")))?;
            let ciphertext: &[u8] = payload
                .get(25..)
                .ok_or_else(|| Error::NotFound(String::from("ciphertext")))?;

            // Compose key
            let shared_key: [u8; 32] = util::generate_shared_key(secret_key, public_key)?;
            let key: Sha256Hash = Sha256Hash::hash(&shared_key);

            // Compose cipher
            let mut cipher = XChaCha20::new(key.as_byte_array().into(), nonce.into());

            // Decrypt
            let mut buffer: Vec<u8> = ciphertext.to_vec();
            cipher.apply_keystream(&mut buffer);

            Ok(buffer)
        }
        Version::V2 => {
            let conversation_key: ConversationKey =
                ConversationKey::derive(secret_key, public_key)?;
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
        let encrypted_content = encrypt(
            alice_keys.secret_key().unwrap(),
            &bob_pk,
            &content,
            Version::V2,
        )
        .unwrap();
        assert_eq!(
            decrypt(
                bob_keys.secret_key().unwrap(),
                &alice_pk,
                &encrypted_content
            )
            .unwrap(),
            content
        );
    }

    #[test]
    fn test_nip44_decryption() {
        let secret_key =
            SecretKey::from_str("0000000000000000000000000000000000000000000000000000000000000002")
                .unwrap();
        let public_key =
            PublicKey::from_str("1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdeb")
                .unwrap();
        let payload =
            "AUXEhLosA5eFMYOtumkiFW4Joq1OPmkU8k/25+3+VDFvOU39qkUDl1aiy8Q+0ozTwbhD57VJoIYayYS++hE=";
        assert_eq!(
            decrypt(&secret_key, &public_key, payload).unwrap(),
            String::from("A Peer-to-Peer Electronic Cash System")
        );

        let secret_key =
            SecretKey::from_str("0000000000000000000000000000000000000000000000000000000000000001")
                .unwrap();
        let public_key =
            PublicKey::from_str("79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798")
                .unwrap();
        let payload = "AdYN4IQFz5veUIFH6CIkrGr0CcErnlSS4VdvoQaP2DCB1dIFL72HSriG1aFABcTlu86hrsG0MdOO9rPdVXc3jptMMzqvIN6tJlHPC8GdwFD5Y8BT76xIIOTJR2W0IdrM7++WC/9harEJAdeWHDAC9zNJX81CpCz4fnV1FZ8GxGLC0nUF7NLeUiNYu5WFXQuO9uWMK0pC7tk3XVogk90X6rwq0MQG9ihT7e1elatDy2YGat+VgQlDrz8ZLRw/lvU+QqeXMQgjqn42sMTrimG6NdKfHJSVWkT6SKZYVsuTyU1Iu5Nk0twEV8d11/MPfsMx4i36arzTC9qxE6jftpOoG8f/jwPTSCEpHdZzrb/CHJcpc+zyOW9BZE2ZOmSxYHAE0ustC9zRNbMT3m6LqxIoHq8j+8Ysu+Cwqr4nUNLYq/Q31UMdDg1oamYS17mWIAS7uf2yF5uT5IlG";
        assert_eq!(decrypt(&secret_key, &public_key, payload).unwrap(), String::from("A purely peer-to-peer version of electronic cash would allow online payments to be sent directly from one party to another without going through a financial institution. Digital signatures provide part of the solution, but the main benefits are lost if a trusted third party is still required to prevent double-spending."));

        let secret_key =
            SecretKey::from_str("fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364139")
                .unwrap();
        let public_key =
            PublicKey::from_str("0000000000000000000000000000000000000000000000000000000000000002")
                .unwrap();
        let payload = "AfSBdQ4T36kLcit8zg2znYCw2y6JXMMAGjM=";
        assert_eq!(
            decrypt(&secret_key, &public_key, payload).unwrap(),
            String::from("a")
        );
    }
}
