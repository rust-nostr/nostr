use aes::Aes256;
use base64::{decode, encode};
use block_modes::{block_padding, BlockMode, Cbc};
use secp256k1::{ecdh, rand::random, PublicKey, SecretKey, XOnlyPublicKey};
use std::convert::From;
use std::str::FromStr;
use thiserror::Error;

type Aes256Cbc = Cbc<Aes256, block_padding::Pkcs7>;

#[derive(Error, Debug, PartialEq)]
pub enum DecryptError {
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
}

pub fn encrypt(sk: &SecretKey, pk: &XOnlyPublicKey, text: &str) -> String {
    let key = generate_shared_key(sk, pk);
    let iv: [u8; 16] = random();
    // This shouldn't fail because we've already validated the inputs
    let cipher = Aes256Cbc::new_from_slices(&key, &iv).expect("Invalid arguments to Aes");
    let cipher_text = cipher.encrypt_vec(text.as_bytes());
    format!("{}?iv={}", encode(cipher_text), encode(iv))
}

pub fn decrypt(
    sk: &SecretKey,
    pk: &XOnlyPublicKey,
    encrypted_content: &str,
) -> Result<String, DecryptError> {
    let parsed_content: Vec<&str> = encrypted_content.split("?iv=").collect();
    if parsed_content.len() != 2 {
        return Err(DecryptError::InvalidContentFormat);
    }
    let encrypted_content =
        decode(parsed_content[0]).map_err(|_| DecryptError::Base64DecodeError)?;
    let iv = decode(parsed_content[1]).map_err(|_| DecryptError::Base64DecodeError)?;
    let key = generate_shared_key(sk, pk);
    // This shouldn't fail because we've already validated the inputs
    let cipher = Aes256Cbc::new_from_slices(&key, &iv).expect("Invalid arguments to Aes");
    let decryptedtext = cipher
        .decrypt_vec(&encrypted_content)
        .map_err(|_| DecryptError::WrongBlockMode)?;
    String::from_utf8(decryptedtext).map_err(|_| DecryptError::Utf8EncodeError)
}

fn generate_shared_key(sk: &SecretKey, pk: &XOnlyPublicKey) -> Vec<u8> {
    let pk_normalized = from_schnorr_pk(pk);
    ecdh::SharedSecret::new_with_hash(&pk_normalized, sk, |x, _| x.into()).to_vec()
}

fn from_schnorr_pk(schnorr_pk: &XOnlyPublicKey) -> PublicKey {
    let mut pk = String::from("02");
    pk.push_str(&schnorr_pk.to_string());
    PublicKey::from_str(&pk).expect("Failed to make a PublicKey with the addition of 02")
}

#[cfg(test)]
mod tests {

    use super::*;
    use secp256k1::{KeyPair, Secp256k1};
    use std::error::Error;

    type TestResult = Result<(), Box<dyn Error>>;

    #[test]
    fn test_encryption_decryption() -> TestResult {
        let secp = Secp256k1::new();

        let sender_sk = SecretKey::from_str(
            "6b911fd37cdf5c81d4c0adb1ab7fa822ed253ab0ad9aa18d77257c88b29b718e",
        )?;
        let sender_key_pair = KeyPair::from_secret_key(&secp, sender_sk);
        let sender_pk = XOnlyPublicKey::from_keypair(&sender_key_pair);

        let receiver_sk = SecretKey::from_str(
            "7b911fd37cdf5c81d4c0adb1ab7fa822ed253ab0ad9aa18d77257c88b29b718e",
        )?;
        let receiver_key_pair = KeyPair::from_secret_key(&secp, receiver_sk);
        let receiver_pk = XOnlyPublicKey::from_keypair(&receiver_key_pair);

        let encrypted_content_from_outside =
            "dJc+WbBgaFCD2/kfg1XCWJParplBDxnZIdJGZ6FCTOg=?iv=M6VxRPkMZu7aIdD+10xPuw==";

        let content = String::from("Saturn, bringer of old age");

        let encrypted_content = encrypt(&sender_sk, &receiver_pk, &content);

        assert_eq!(
            decrypt(&receiver_sk, &sender_pk, &encrypted_content)?,
            content
        );

        assert_eq!(
            decrypt(&receiver_sk, &sender_pk, &encrypted_content_from_outside)?,
            content
        );

        assert_eq!(
            decrypt(&sender_sk, &receiver_pk, "invalidcontentformat").unwrap_err(),
            DecryptError::InvalidContentFormat
        );
        assert_eq!(
            decrypt(&sender_sk, &receiver_pk, "badbase64?iv=encode").unwrap_err(),
            DecryptError::Base64DecodeError
        );

        //Content encrypted with aes256 using GCM mode
        assert_eq!(
            decrypt(
                &sender_sk,
                &receiver_pk,
                "nseh0cQPEFID5C0CxYdcPwp091NhRQ==?iv=8PHy8/T19vf4+fr7/P3+/w=="
            )
            .unwrap_err(),
            DecryptError::WrongBlockMode
        );

        Ok(())
    }
}
