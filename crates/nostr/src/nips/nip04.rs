// Copyright (c) 2021 Paul Miller
// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP04: Encrypted Direct Message (deprecated in favor of NIP17)
//!
//! <div class="warning"><strong>Unsecure!</strong> Deprecated in favor of NIP17!</div>
//!
//! <https://github.com/nostr-protocol/nips/blob/master/04.md>

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use aes::cipher::block_padding::Pkcs7;
use aes::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use aes::Aes256;
use base64::engine::{general_purpose, Engine};
#[cfg(feature = "std")]
use bitcoin::secp256k1::rand;
use bitcoin::secp256k1::rand::RngCore;
use cbc::{Decryptor, Encryptor};

use crate::{util, PublicKey, SecretKey};

type Aes256CbcEnc = Encryptor<Aes256>;
type Aes256CbcDec = Decryptor<Aes256>;

/// `NIP04` error
#[derive(Debug, Eq, PartialEq)]
pub enum Error {
    /// Invalid content format
    InvalidContentFormat,
    /// Error while decoding from base64
    Base64Decode,
    /// Error while encoding to UTF-8
    Utf8Encode,
    /// Wrong encryption block mode
    WrongBlockMode,
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidContentFormat => write!(f, "Invalid NIP04 content format"),
            Self::Base64Decode => write!(f, "Error while decoding NIP04 from base64"),
            Self::Utf8Encode => write!(f, "Error while encoding NIP04 to UTF-8"),
            Self::WrongBlockMode => write!(
                f,
                "Wrong encryption block mode. The content must be encrypted using CBC mode!"
            ),
        }
    }
}

/// Encrypt
///
/// <div class="warning"><strong>Unsecure!</strong> Deprecated in favor of NIP17!</div>
#[inline]
#[cfg(feature = "std")]
pub fn encrypt<T>(
    secret_key: &SecretKey,
    public_key: &PublicKey,
    content: T,
) -> Result<String, Error>
where
    T: AsRef<[u8]>,
{
    encrypt_with_rng(&mut rand::thread_rng(), secret_key, public_key, content)
}

/// Encrypt
///
/// <div class="warning"><strong>Unsecure!</strong> Deprecated in favor of NIP17!</div>
pub fn encrypt_with_rng<R, T>(
    rng: &mut R,
    secret_key: &SecretKey,
    public_key: &PublicKey,
    content: T,
) -> Result<String, Error>
where
    R: RngCore,
    T: AsRef<[u8]>,
{
    // Generate key
    let key: [u8; 32] = util::generate_shared_key(secret_key, public_key);

    // Generate iv
    let mut iv: [u8; 16] = [0u8; 16];
    rng.fill_bytes(&mut iv);

    // Compose cipher
    let cipher = Aes256CbcEnc::new(&key.into(), &iv.into());

    // Encrypt
    let result: Vec<u8> = cipher.encrypt_padded_vec_mut::<Pkcs7>(content.as_ref());

    // Encode with base64
    Ok(format!(
        "{}?iv={}",
        general_purpose::STANDARD.encode(result),
        general_purpose::STANDARD.encode(iv)
    ))
}

/// Decrypts content to bytes
///
/// <div class="warning"><strong>Unsecure!</strong> Deprecated in favor of NIP17!</div>
pub fn decrypt_to_bytes<S>(
    secret_key: &SecretKey,
    public_key: &PublicKey,
    encrypted_content: S,
) -> Result<Vec<u8>, Error>
where
    S: Into<String>,
{
    let encrypted_content: String = encrypted_content.into();
    let parsed_content: Vec<&str> = encrypted_content.split("?iv=").collect();
    if parsed_content.len() != 2 {
        return Err(Error::InvalidContentFormat);
    }

    let encrypted_content: Vec<u8> = general_purpose::STANDARD
        .decode(parsed_content[0])
        .map_err(|_| Error::Base64Decode)?;
    let iv: Vec<u8> = general_purpose::STANDARD
        .decode(parsed_content[1])
        .map_err(|_| Error::Base64Decode)?;
    let key: [u8; 32] = util::generate_shared_key(secret_key, public_key);

    let cipher = Aes256CbcDec::new(&key.into(), iv.as_slice().into());
    let result = cipher
        .decrypt_padded_vec_mut::<Pkcs7>(&encrypted_content)
        .map_err(|_| Error::WrongBlockMode)?;

    Ok(result)
}

/// Decrypts content to a UTF-8 string
///
/// <div class="warning"><strong>Unsecure!</strong> Deprecated in favor of NIP17!</div>
#[inline]
pub fn decrypt<T>(
    secret_key: &SecretKey,
    public_key: &PublicKey,
    encrypted_content: T,
) -> Result<String, Error>
where
    T: Into<String>,
{
    let result = decrypt_to_bytes(secret_key, public_key, encrypted_content)?;
    String::from_utf8(result).map_err(|_| Error::Utf8Encode)
}

#[cfg(test)]
#[cfg(feature = "std")]
mod tests {
    use core::str::FromStr;

    use super::*;
    use crate::Keys;

    #[test]
    fn test_encryption_decryption() {
        let sender_sk =
            SecretKey::from_str("6b911fd37cdf5c81d4c0adb1ab7fa822ed253ab0ad9aa18d77257c88b29b718e")
                .unwrap();
        let sender_keys = Keys::new(sender_sk);
        let sender_pk = sender_keys.public_key();

        let receiver_sk =
            SecretKey::from_str("7b911fd37cdf5c81d4c0adb1ab7fa822ed253ab0ad9aa18d77257c88b29b718e")
                .unwrap();
        let receiver_keys = Keys::new(receiver_sk);
        let receiver_pk = receiver_keys.public_key();

        let encrypted_content_from_outside =
            "dJc+WbBgaFCD2/kfg1XCWJParplBDxnZIdJGZ6FCTOg=?iv=M6VxRPkMZu7aIdD+10xPuw==";

        let content = String::from("Saturn, bringer of old age");

        let encrypted_content = encrypt(sender_keys.secret_key(), &receiver_pk, &content).unwrap();

        assert_eq!(
            decrypt(receiver_keys.secret_key(), &sender_pk, encrypted_content).unwrap(),
            content
        );

        assert_eq!(
            decrypt(
                receiver_keys.secret_key(),
                &sender_pk,
                encrypted_content_from_outside
            )
            .unwrap(),
            content
        );

        assert_eq!(
            decrypt(
                sender_keys.secret_key(),
                &receiver_pk,
                "invalidcontentformat"
            )
            .unwrap_err(),
            Error::InvalidContentFormat
        );
        assert_eq!(
            decrypt(
                sender_keys.secret_key(),
                &receiver_pk,
                "badbase64?iv=encode"
            )
            .unwrap_err(),
            Error::Base64Decode
        );

        // Content encrypted with aes256 using GCM mode
        assert_eq!(
            decrypt(
                sender_keys.secret_key(),
                &receiver_pk,
                "nseh0cQPEFID5C0CxYdcPwp091NhRQ==?iv=8PHy8/T19vf4+fr7/P3+/w=="
            )
            .unwrap_err(),
            Error::WrongBlockMode
        );
    }
}
