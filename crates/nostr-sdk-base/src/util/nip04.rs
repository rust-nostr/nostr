// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::convert::From;
use std::str::FromStr;

use aes::cipher::block_padding::Pkcs7;
use aes::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use aes::Aes256;
use cbc::{Decryptor, Encryptor};
use secp256k1::{ecdh, PublicKey, SecretKey, XOnlyPublicKey};
use thiserror::Error;

type Aes256CbcEnc = Encryptor<Aes256>;
type Aes256CbcDec = Decryptor<Aes256>;

#[derive(Error, Debug, Eq, PartialEq)]
pub enum Error {
    #[error(
        r#"Invalid content format. Expected format "<encrypted_text>?iv=<initialization_vec>""#
    )]
    InvalidContentFormat,
    #[error("Error while decoding from base64")]
    Base64DecodeError,
    #[error("Error while encoding to UTF-8")]
    Utf8EncodeError,
    #[error("Wrong encryption block mode.The content must be encrypted using CBC mode!")]
    WrongBlockMode,
    #[error("Secp256k1 error: {}", _0)]
    Secp256k1Error(secp256k1::Error),
}

impl From<secp256k1::Error> for Error {
    fn from(err: secp256k1::Error) -> Self {
        Self::Secp256k1Error(err)
    }
}

pub fn encrypt(sk: &SecretKey, pk: &XOnlyPublicKey, text: &str) -> Result<String, Error> {
    let key: Vec<u8> = generate_shared_key(sk, pk)?;
    let iv: [u8; 16] = secp256k1::rand::random();

    let cipher = Aes256CbcEnc::new(key.as_slice().into(), &iv.into());
    let result: Vec<u8> = cipher.encrypt_padded_vec_mut::<Pkcs7>(text.as_bytes());

    Ok(format!(
        "{}?iv={}",
        base64::encode(result),
        base64::encode(iv)
    ))
}

pub fn decrypt(
    sk: &SecretKey,
    pk: &XOnlyPublicKey,
    encrypted_content: &str,
) -> Result<String, Error> {
    let parsed_content: Vec<&str> = encrypted_content.split("?iv=").collect();
    if parsed_content.len() != 2 {
        return Err(Error::InvalidContentFormat);
    }

    let encrypted_content: Vec<u8> =
        base64::decode(parsed_content[0]).map_err(|_| Error::Base64DecodeError)?;
    let iv: Vec<u8> = base64::decode(parsed_content[1]).map_err(|_| Error::Base64DecodeError)?;
    let key: Vec<u8> = generate_shared_key(sk, pk)?;

    let cipher = Aes256CbcDec::new(key.as_slice().into(), iv.as_slice().into());
    let result = cipher
        .decrypt_padded_vec_mut::<Pkcs7>(&encrypted_content)
        .map_err(|_| Error::WrongBlockMode)?;

    String::from_utf8(result).map_err(|_| Error::Utf8EncodeError)
}

fn generate_shared_key(sk: &SecretKey, pk: &XOnlyPublicKey) -> Result<Vec<u8>, Error> {
    let pk_normalized: PublicKey = from_schnorr_pk(pk)?;
    let ssp = ecdh::shared_secret_point(&pk_normalized, sk);

    let mut shared_key = [0u8; 32];
    shared_key.copy_from_slice(&ssp[..32]);
    Ok(shared_key.to_vec())
}

fn from_schnorr_pk(schnorr_pk: &XOnlyPublicKey) -> Result<PublicKey, Error> {
    let mut pk = String::from("02");
    pk.push_str(&schnorr_pk.to_string());
    Ok(PublicKey::from_str(&pk)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    use secp256k1::{KeyPair, Secp256k1};

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    #[test]
    fn test_encryption_decryption() -> TestResult {
        let secp = Secp256k1::new();

        let sender_sk = SecretKey::from_str(
            "6b911fd37cdf5c81d4c0adb1ab7fa822ed253ab0ad9aa18d77257c88b29b718e",
        )?;
        let sender_key_pair = KeyPair::from_secret_key(&secp, &sender_sk);
        let sender_pk = XOnlyPublicKey::from_keypair(&sender_key_pair).0;

        let receiver_sk = SecretKey::from_str(
            "7b911fd37cdf5c81d4c0adb1ab7fa822ed253ab0ad9aa18d77257c88b29b718e",
        )?;
        let receiver_key_pair = KeyPair::from_secret_key(&secp, &receiver_sk);
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
            Error::Base64DecodeError
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
