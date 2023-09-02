// Copyright (c) 2021 Paul Miller
// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! NIP04
//!
//! <https://github.com/nostr-protocol/nips/blob/master/04.md>

use alloc::string::String;
use alloc::vec::Vec;
use core::convert::From;
use core::fmt;

use aes::cipher::block_padding::Pkcs7;
use aes::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use aes::Aes256;
use base64::engine::{general_purpose, Engine};
#[cfg(feature = "std")]
use bitcoin::secp256k1::rand;
use bitcoin::secp256k1::rand::RngCore;
use bitcoin::secp256k1::{self, SecretKey, XOnlyPublicKey};
use cbc::{Decryptor, Encryptor};

use crate::util;

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
    /// Secp256k1 error
    Secp256k1(secp256k1::Error),
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidContentFormat => write!(f, "Invalid content format"),
            Self::Base64Decode => write!(f, "Error while decoding from base64"),
            Self::Utf8Encode => write!(f, "Error while encoding to UTF-8"),
            Self::WrongBlockMode => write!(
                f,
                "Wrong encryption block mode. The content must be encrypted using CBC mode!"
            ),
            Self::Secp256k1(e) => write!(f, "Secp256k1: {e}"),
        }
    }
}

impl From<secp256k1::Error> for Error {
    fn from(e: secp256k1::Error) -> Self {
        Self::Secp256k1(e)
    }
}

/// Entrypt
#[cfg(feature = "std")]
pub fn encrypt<T>(sk: &SecretKey, pk: &XOnlyPublicKey, text: T) -> Result<String, Error>
where
    T: AsRef<[u8]>,
{
    encrypt_with_rng(&mut rand::thread_rng(), sk, pk, text)
}

/// Entrypt
pub fn encrypt_with_rng<R, T>(
    rng: &mut R,
    sk: &SecretKey,
    pk: &XOnlyPublicKey,
    text: T,
) -> Result<String, Error>
where
    R: RngCore,
    T: AsRef<[u8]>,
{
    // Generate key
    let key: [u8; 32] = util::generate_shared_key(sk, pk)?;

    // Generate iv
    let mut iv: [u8; 16] = [0u8; 16];
    rng.fill_bytes(&mut iv);

    // Compose cipher
    let cipher = Aes256CbcEnc::new(&key.into(), &iv.into());

    // Encrypt
    let result: Vec<u8> = cipher.encrypt_padded_vec_mut::<Pkcs7>(text.as_ref());

    // Encode with base64
    Ok(format!(
        "{}?iv={}",
        general_purpose::STANDARD.encode(result),
        general_purpose::STANDARD.encode(iv)
    ))
}

/// Dectypt
pub fn decrypt<S>(
    sk: &SecretKey,
    pk: &XOnlyPublicKey,
    encrypted_content: S,
) -> Result<String, Error>
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
    let key: [u8; 32] = util::generate_shared_key(sk, pk)?;

    let cipher = Aes256CbcDec::new(&key.into(), iv.as_slice().into());
    let result = cipher
        .decrypt_padded_vec_mut::<Pkcs7>(&encrypted_content)
        .map_err(|_| Error::WrongBlockMode)?;

    String::from_utf8(result).map_err(|_| Error::Utf8Encode)
}

#[cfg(test)]
#[cfg(feature = "std")]
mod tests {
    use core::str::FromStr;

    use bitcoin::secp256k1::{KeyPair, Secp256k1};

    use super::*;

    #[test]
    fn test_encryption_decryption() {
        let secp = Secp256k1::new();

        let sender_sk =
            SecretKey::from_str("6b911fd37cdf5c81d4c0adb1ab7fa822ed253ab0ad9aa18d77257c88b29b718e")
                .unwrap();
        let sender_key_pair = KeyPair::from_secret_key(&secp, &sender_sk);
        let sender_pk = XOnlyPublicKey::from_keypair(&sender_key_pair).0;

        let receiver_sk =
            SecretKey::from_str("7b911fd37cdf5c81d4c0adb1ab7fa822ed253ab0ad9aa18d77257c88b29b718e")
                .unwrap();
        let receiver_key_pair = KeyPair::from_secret_key(&secp, &receiver_sk);
        let receiver_pk = XOnlyPublicKey::from_keypair(&receiver_key_pair).0;

        let encrypted_content_from_outside =
            "dJc+WbBgaFCD2/kfg1XCWJParplBDxnZIdJGZ6FCTOg=?iv=M6VxRPkMZu7aIdD+10xPuw==";

        let content = String::from("Saturn, bringer of old age");

        let encrypted_content = encrypt(&sender_sk, &receiver_pk, &content).unwrap();

        assert_eq!(
            decrypt(&receiver_sk, &sender_pk, &encrypted_content).unwrap(),
            content
        );

        assert_eq!(
            decrypt(&receiver_sk, &sender_pk, encrypted_content_from_outside).unwrap(),
            content
        );

        assert_eq!(
            decrypt(&sender_sk, &receiver_pk, "invalidcontentformat").unwrap_err(),
            Error::InvalidContentFormat
        );
        assert_eq!(
            decrypt(&sender_sk, &receiver_pk, "badbase64?iv=encode").unwrap_err(),
            Error::Base64Decode
        );

        //Content encrypted with aes256 using GCM mode
        assert_eq!(
            decrypt(
                &sender_sk,
                &receiver_pk,
                "nseh0cQPEFID5C0CxYdcPwp091NhRQ==?iv=8PHy8/T19vf4+fr7/P3+/w=="
            )
            .unwrap_err(),
            Error::WrongBlockMode
        );
    }
}
