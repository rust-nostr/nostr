// Copyright (c) 2021 Paul Miller
// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! NIP04
//!
//! <https://github.com/nostr-protocol/nips/blob/master/04.md>

use core::convert::From;
#[cfg(all(feature = "alloc", not(feature = "std")))]
use core::error::Error as StdError;
use core::fmt;
use core::str::FromStr;
#[cfg(feature = "std")]
use std::error::Error as StdError;

use aes::cipher::block_padding::Pkcs7;
use aes::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use aes::Aes256;
use base64::engine::{general_purpose, Engine};
use cbc::{Decryptor, Encryptor};
use secp256k1::{ecdh, rand, PublicKey, SecretKey, XOnlyPublicKey};

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

impl StdError for Error {}

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
            Self::Secp256k1(e) => write!(f, "secp256k1 error: {e}"),
        }
    }
}

impl From<secp256k1::Error> for Error {
    fn from(e: secp256k1::Error) -> Self {
        Self::Secp256k1(e)
    }
}

/// Entrypt
pub fn encrypt<T>(sk: &SecretKey, pk: &XOnlyPublicKey, text: T) -> Result<String, Error>
where
    T: AsRef<[u8]>,
{
    let key: [u8; 32] = generate_shared_key(sk, pk)?;
    let iv: [u8; 16] = rand::random();

    let cipher = Aes256CbcEnc::new(&key.into(), &iv.into());
    let result: Vec<u8> = cipher.encrypt_padded_vec_mut::<Pkcs7>(text.as_ref());

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
    let key: [u8; 32] = generate_shared_key(sk, pk)?;

    let cipher = Aes256CbcDec::new(&key.into(), iv.as_slice().into());
    let result = cipher
        .decrypt_padded_vec_mut::<Pkcs7>(&encrypted_content)
        .map_err(|_| Error::WrongBlockMode)?;

    String::from_utf8(result).map_err(|_| Error::Utf8Encode)
}

/// Generate shared key
pub fn generate_shared_key(sk: &SecretKey, pk: &XOnlyPublicKey) -> Result<[u8; 32], Error> {
    let pk_normalized: PublicKey = normalize_schnorr_pk(pk)?;
    let ssp = ecdh::shared_secret_point(&pk_normalized, sk);
    let mut shared_key: [u8; 32] = [0u8; 32];
    shared_key.copy_from_slice(&ssp[..32]);
    Ok(shared_key)
}

/// Normalize Schnorr public key
fn normalize_schnorr_pk(schnorr_pk: &XOnlyPublicKey) -> Result<PublicKey, Error> {
    let mut pk = String::from("02");
    pk.push_str(&schnorr_pk.to_string());
    Ok(PublicKey::from_str(&pk)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    use secp256k1::KeyPair;

    use crate::{Result, SECP256K1};

    #[test]
    fn test_encryption_decryption() -> Result<()> {
        let sender_sk = SecretKey::from_str(
            "6b911fd37cdf5c81d4c0adb1ab7fa822ed253ab0ad9aa18d77257c88b29b718e",
        )?;
        let sender_key_pair = KeyPair::from_secret_key(&SECP256K1, &sender_sk);
        let sender_pk = XOnlyPublicKey::from_keypair(&sender_key_pair).0;

        let receiver_sk = SecretKey::from_str(
            "7b911fd37cdf5c81d4c0adb1ab7fa822ed253ab0ad9aa18d77257c88b29b718e",
        )?;
        let receiver_key_pair = KeyPair::from_secret_key(&SECP256K1, &receiver_sk);
        let receiver_pk = XOnlyPublicKey::from_keypair(&receiver_key_pair).0;

        let encrypted_content_from_outside =
            "dJc+WbBgaFCD2/kfg1XCWJParplBDxnZIdJGZ6FCTOg=?iv=M6VxRPkMZu7aIdD+10xPuw==";

        let content = String::from("Saturn, bringer of old age");

        let encrypted_content = encrypt(&sender_sk, &receiver_pk, &content).unwrap();

        assert_eq!(
            decrypt(&receiver_sk, &sender_pk, &encrypted_content)?,
            content
        );

        assert_eq!(
            decrypt(&receiver_sk, &sender_pk, encrypted_content_from_outside)?,
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

        Ok(())
    }
}
