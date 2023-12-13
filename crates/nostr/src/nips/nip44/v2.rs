// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP44 (v2)
//!
//! <https://github.com/nostr-protocol/nips/blob/master/44.md>

use alloc::string::{FromUtf8Error, String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use core::array::TryFromSliceError;
use core::ops::Range;
use core::{fmt, iter};

use base64::engine::{general_purpose, Engine};
use bitcoin::hashes::hmac::{Hmac, HmacEngine};
use bitcoin::hashes::sha256::Hash as Sha256Hash;
use bitcoin::hashes::{Hash, HashEngine};
#[cfg(feature = "std")]
use bitcoin::secp256k1::rand::rngs::OsRng;
use bitcoin::secp256k1::rand::RngCore;
use chacha20::cipher::{KeyIvInit, StreamCipher};
use chacha20::ChaCha20;

use super::Error;

const MESSAGE_KEYS_SIZE: usize = 76;
const MESSAGES_KEYS_ENCRYPTION_SIZE: usize = 32;
const MESSAGES_KEYS_NONCE_SIZE: usize = 12;
const MESSAGES_KEYS_ENCRYPTION_RANGE: Range<usize> = 0..MESSAGES_KEYS_ENCRYPTION_SIZE;
const MESSAGES_KEYS_NONCE_RANGE: Range<usize> =
    MESSAGES_KEYS_ENCRYPTION_SIZE..MESSAGES_KEYS_ENCRYPTION_SIZE + MESSAGES_KEYS_NONCE_SIZE;
const MESSAGES_KEYS_AUTH_RANGE: Range<usize> =
    MESSAGES_KEYS_ENCRYPTION_SIZE + MESSAGES_KEYS_NONCE_SIZE..MESSAGE_KEYS_SIZE;

/// Error
#[derive(Debug, PartialEq, Eq)]
pub enum ErrorV2 {
    /// Error while encoding to UTF-8
    Utf8Encode(FromUtf8Error),
    /// Try from slice
    TryFromSlice(String),
    /// HKDF Length
    HkdfLength(usize),
    /// Message is empty
    MessageEmpty,
    /// Message is too long
    MessageTooLong,
    /// Invalid HMAC
    InvalidHmac,
    /// Invalid padding
    InvalidPadding,
}

#[cfg(feature = "std")]
impl std::error::Error for ErrorV2 {}

impl fmt::Display for ErrorV2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Utf8Encode(e) => write!(f, "error while encoding to UTF-8: {e}"),
            Self::TryFromSlice(e) => write!(f, "try from slice error: {e}"),
            Self::HkdfLength(size) => write!(f, "invalid Length for HKDF: {size}"),
            Self::MessageEmpty => write!(f, "message empty"),
            Self::MessageTooLong => write!(f, "message too long"),
            Self::InvalidHmac => write!(f, "invalid HMAC"),
            Self::InvalidPadding => write!(f, "invalid padding"),
        }
    }
}

impl From<FromUtf8Error> for ErrorV2 {
    fn from(e: FromUtf8Error) -> Self {
        Self::Utf8Encode(e)
    }
}

impl From<TryFromSliceError> for ErrorV2 {
    fn from(e: TryFromSliceError) -> Self {
        Self::TryFromSlice(e.to_string())
    }
}

struct MessageKeys([u8; MESSAGE_KEYS_SIZE]);

impl MessageKeys {
    pub fn from_slice(slice: &[u8]) -> Result<Self, TryFromSliceError> {
        Ok(Self(slice.try_into()?))
    }

    #[inline]
    pub fn encryption(&self) -> &[u8] {
        &self.0[MESSAGES_KEYS_ENCRYPTION_RANGE]
    }

    #[inline]
    pub fn nonce(&self) -> &[u8] {
        &self.0[MESSAGES_KEYS_NONCE_RANGE]
    }

    #[inline]
    pub fn auth(&self) -> &[u8] {
        &self.0[MESSAGES_KEYS_AUTH_RANGE]
    }
}

/// Encrypt with NIP44 (v2)
#[cfg(feature = "std")]
pub fn encrypt<T>(shared_key: &[u8; 32], plaintext: T) -> Result<String, Error>
where
    T: AsRef<[u8]>,
{
    encrypt_with_rng(&mut OsRng, shared_key, plaintext)
}

/// Encrypt with NIP44 (v2) using custom Rng
pub fn encrypt_with_rng<R, T>(
    rng: &mut R,
    shared_key: &[u8; 32],
    plaintext: T,
) -> Result<String, Error>
where
    R: RngCore,
    T: AsRef<[u8]>,
{
    // Generate salt
    let mut salt: [u8; 32] = [0; 32];
    rng.fill_bytes(&mut salt);

    // Get Message Keys
    let keys: MessageKeys = get_message_keys(shared_key, &salt)?;

    // Pad
    let mut buffer: Vec<u8> = pad(plaintext)?;

    // Compose cipher and encrypt
    let mut cipher = ChaCha20::new(keys.encryption().into(), keys.nonce().into());
    cipher.apply_keystream(&mut buffer);

    // HMAC-SHA256
    let mut engine: HmacEngine<Sha256Hash> = HmacEngine::new(keys.auth());
    engine.input(&buffer);
    let hmac: [u8; 32] = Hmac::from_engine(engine).to_byte_array();

    // Compose payload
    let mut payload: Vec<u8> = vec![2]; // Version
    payload.extend_from_slice(&salt);
    payload.extend_from_slice(&buffer);
    payload.extend_from_slice(&hmac);

    // Encode payload to base64
    Ok(general_purpose::STANDARD.encode(payload))
}

/// Decrypt with NIP44 (v2)
///
/// The payload MUST be already decoded from base64
pub fn decrypt<T>(shared_key: &[u8; 32], payload: T) -> Result<String, Error>
where
    T: AsRef<[u8]>,
{
    // Get data from payload
    let payload: &[u8] = payload.as_ref();
    let len: usize = payload.len();
    let salt: &[u8] = payload
        .get(1..33)
        .ok_or_else(|| Error::NotFound(String::from("salt")))?;
    let buffer: &[u8] = payload
        .get(33..len - 32)
        .ok_or_else(|| Error::NotFound(String::from("buffer")))?;
    let mac: &[u8] = payload
        .get(len - 32..)
        .ok_or_else(|| Error::NotFound(String::from("hmac")))?;

    // Compose Message Keys
    let keys: MessageKeys = get_message_keys(shared_key, salt)?;

    // Check HMAC-SHA256
    let mut engine: HmacEngine<Sha256Hash> = HmacEngine::new(keys.auth());
    engine.input(buffer);
    let calculated_mac: [u8; 32] = Hmac::from_engine(engine).to_byte_array();
    if mac != calculated_mac.as_slice() {
        return Err(ErrorV2::InvalidHmac.into());
    }

    // Compose cipher
    let mut cipher = ChaCha20::new(keys.encryption().into(), keys.nonce().into());
    let mut buffer: Vec<u8> = buffer.to_vec();
    cipher.apply_keystream(&mut buffer);

    let be_bytes: [u8; 2] = buffer[0..2]
        .try_into()
        .map_err(|e| Error::from(ErrorV2::from(e)))?;
    let unpadded_len: usize = u16::from_be_bytes(be_bytes) as usize;
    let unpadded: &[u8] = &buffer[2..2 + unpadded_len];
    if unpadded.is_empty() {
        return Err(ErrorV2::MessageEmpty.into());
    }

    if unpadded.len() != unpadded_len {
        return Err(ErrorV2::InvalidPadding.into());
    }

    if buffer.len() != 2 + calc_padding(unpadded_len) {
        return Err(ErrorV2::InvalidPadding.into());
    }

    String::from_utf8(unpadded.to_vec()).map_err(|e| Error::V2(ErrorV2::from(e)))
}

fn get_message_keys(shared_key: &[u8; 32], salt: &[u8]) -> Result<MessageKeys, ErrorV2> {
    let prk: [u8; 32] = hkdf_extract(salt, shared_key);
    let expanded_key = hkdf_expand(&prk, b"nip44-v2", MESSAGE_KEYS_SIZE);
    MessageKeys::from_slice(&expanded_key).map_err(|_| ErrorV2::HkdfLength(expanded_key.len()))
}

fn pad<T>(unpadded: T) -> Result<Vec<u8>, ErrorV2>
where
    T: AsRef<[u8]>,
{
    let unpadded: &[u8] = unpadded.as_ref();
    let len: usize = unpadded.len();

    if len < 1 {
        return Err(ErrorV2::MessageEmpty);
    }

    if len > 65536 - 128 {
        return Err(ErrorV2::MessageTooLong);
    }

    let take: usize = calc_padding(len) - len;
    let mut padded: Vec<u8> = Vec::with_capacity(2 + len + take);
    padded.extend_from_slice(&(len as u16).to_be_bytes());
    padded.extend_from_slice(unpadded);
    padded.extend(iter::repeat(0).take(take));
    Ok(padded)
}

#[inline]
fn calc_padding(len: usize) -> usize {
    if len <= 32 {
        return 32;
    }
    let nextpower: usize = 1 << (log2_round_down(len - 1) + 1);
    let chunk: usize = if nextpower <= 256 { 32 } else { nextpower / 8 };
    chunk * (((len - 1) / chunk) + 1)
}

/// Returns the base 2 logarithm of the number, rounded down.
#[inline]
fn log2_round_down(x: usize) -> u32 {
    if x == 0 {
        0
    } else {
        let x: f64 = x as f64;
        x.log2().floor() as u32
    }
}

fn hkdf_extract(salt: &[u8], input_key_material: &[u8]) -> [u8; 32] {
    let mut engine: HmacEngine<Sha256Hash> = HmacEngine::new(salt);
    engine.input(input_key_material);
    Hmac::from_engine(engine).to_byte_array()
}

fn hkdf_expand(prk: &[u8], info: &[u8], output_len: usize) -> Vec<u8> {
    let mut output = Vec::with_capacity(output_len);
    let mut t = Vec::with_capacity(32);

    let mut i: u8 = 1u8;
    while output.len() < output_len {
        let mut engine: HmacEngine<Sha256Hash> = HmacEngine::new(prk);

        if !t.is_empty() {
            engine.input(&t);
        }

        engine.input(info);
        engine.input(&[i]);

        t = Hmac::from_engine(engine).to_byte_array().to_vec();
        output.extend_from_slice(&t);

        i += 1;
    }

    output.truncate(output_len);
    output
}

#[cfg(test)]
#[cfg(feature = "std")]
mod tests {
    #![allow(dead_code)]

    use core::str::FromStr;

    use bitcoin::secp256k1::{Secp256k1, SecretKey, XOnlyPublicKey};

    use super::*;
    use crate::nips::nip44;
    use crate::util;

    fn val(c: u8, idx: usize) -> u8 {
        match c {
            b'A'..=b'F' => c - b'A' + 10,
            b'a'..=b'f' => c - b'a' + 10,
            b'0'..=b'9' => c - b'0',
            _ => panic!("Invalid character {} at position {}", c as char, idx),
        }
    }

    pub fn hex_decode<T>(hex: T) -> Vec<u8>
    where
        T: AsRef<[u8]>,
    {
        let hex = hex.as_ref();
        let len = hex.len();

        if len % 2 != 0 {
            panic!("Odd number of digits");
        }

        let mut bytes: Vec<u8> = Vec::with_capacity(len / 2);

        for i in (0..len).step_by(2) {
            let high = val(hex[i], i);
            let low = val(hex[i + 1], i + 1);
            bytes.push(high << 4 | low);
        }

        bytes
    }

    #[derive(Debug)]
    struct ValidSec {
        sec1: &'static str,
        sec2: &'static str,
        shared: &'static str,
        salt: &'static str,
        plaintext: &'static str,
        ciphertext: &'static str,
        note: &'static str,
    }

    #[derive(Debug)]
    struct ValidSecParsed {
        sec1: SecretKey,
        sec2: SecretKey,
        shared: [u8; 32],
        salt: [u8; 32],
        plaintext: &'static str,
        ciphertext: &'static str,
        note: &'static str,
    }

    impl ValidSec {
        fn parsed(self) -> ValidSecParsed {
            ValidSecParsed {
                sec1: SecretKey::from_str(self.sec1).unwrap(),
                sec2: SecretKey::from_str(self.sec2).unwrap(),
                shared: hex_decode(self.shared).try_into().unwrap(),
                salt: hex_decode(self.salt).try_into().unwrap(),
                plaintext: self.plaintext,
                ciphertext: self.ciphertext,
                note: self.note,
            }
        }
    }

    #[derive(Debug)]
    struct ValidPub {
        sec1: &'static str,
        pub2: &'static str,
        shared: &'static str,
        salt: &'static str,
        plaintext: &'static str,
        ciphertext: &'static str,
        note: &'static str,
    }

    #[derive(Debug)]
    struct ValidPubParsed {
        sec1: SecretKey,
        pub2: XOnlyPublicKey,
        shared: [u8; 32],
        salt: [u8; 32],
        plaintext: &'static str,
        ciphertext: &'static str,
        note: &'static str,
    }

    impl ValidPub {
        fn parsed(self) -> ValidPubParsed {
            ValidPubParsed {
                sec1: SecretKey::from_str(&self.sec1).unwrap(),
                pub2: XOnlyPublicKey::from_str(&self.pub2).unwrap(),
                shared: hex_decode(self.shared).try_into().unwrap(),
                salt: hex_decode(self.salt).try_into().unwrap(),
                plaintext: self.plaintext,
                ciphertext: self.ciphertext,
                note: self.note,
            }
        }
    }

    #[derive(Debug)]
    struct InvalidPub {
        sec1: &'static str,
        pub2: &'static str,
        shared: &'static str,
        salt: &'static str,
        plaintext: &'static str,
        ciphertext: &'static str,
        note: &'static str,
        error: Error,
    }

    #[derive(Debug)]
    struct InvalidPubParsed {
        sec1: SecretKey,
        pub2: XOnlyPublicKey,
        shared: [u8; 32],
        plaintext: &'static str,
        ciphertext: &'static str,
        note: &'static str,
        error: Error,
    }

    #[derive(Debug)]
    struct InvalidKeys {
        sec1: &'static str,
        pub2: &'static str,
        note: &'static str,
    }

    impl InvalidPub {
        fn parsed(self) -> InvalidPubParsed {
            InvalidPubParsed {
                sec1: SecretKey::from_str(&self.sec1).unwrap(),
                pub2: XOnlyPublicKey::from_str(&self.pub2).unwrap(),
                shared: hex_decode(self.shared).try_into().unwrap(),
                plaintext: self.plaintext,
                ciphertext: self.ciphertext,
                note: self.note,
                error: self.error,
            }
        }
    }

    const VALID_SEC: [ValidSec; 10] = [
        ValidSec {
            sec1: "0000000000000000000000000000000000000000000000000000000000000001",
            sec2: "0000000000000000000000000000000000000000000000000000000000000002",
            shared: "c6047f9441ed7d6d3045406e95c07cd85c778e4b8cef3ca7abac09b95c709ee5",
            salt: "0000000000000000000000000000000000000000000000000000000000000001",
            plaintext: "a",
            ciphertext: "AgAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABYNpT9ESckRbRUY7bUF5P+1rObpA4BNoksAUQ8myMDd9/37W/J2YHvBpRjvy9uC0+ovbpLc0WLaMFieqAMdIYqR14",
            note: "sk1 = 1, sk2 = random, 0x02",
        },
        ValidSec {
            sec1: "0000000000000000000000000000000000000000000000000000000000000002",
            sec2: "0000000000000000000000000000000000000000000000000000000000000001",
            shared: "c6047f9441ed7d6d3045406e95c07cd85c778e4b8cef3ca7abac09b95c709ee5",
            salt: "f00000000000000000000000000000f00000000000000000000000000000000f",
            plaintext: "🍕🫃",
            ciphertext: "AvAAAAAAAAAAAAAAAAAAAPAAAAAAAAAAAAAAAAAAAAAPKY68BwdF7PIT205jBoaZHSs7OMpKsULW5F5ClOJWiy6XjZy7s2v85KugYmbBKgEC2LytbXbxkr7Jpgfk529K3/pP",
            note: "sk1 = 1, sk2 = random, 0x02"
        },
        ValidSec {
            sec1: "5c0c523f52a5b6fad39ed2403092df8cebc36318b39383bca6c00808626fab3a",
            sec2: "4b22aa260e4acb7021e32f38a6cdf4b673c6a277755bfce287e370c924dc936d",
            shared: "94da47d851b9c1ed33b3b72f35434f56aa608d60e573e9c295f568011f4f50a4",
            salt: "b635236c42db20f021bb8d1cdff5ca75dd1a0cc72ea742ad750f33010b24f73b",
            plaintext: "表ポあA鷗ŒéＢ逍Üßªąñ丂㐀𠀀",
            ciphertext: "ArY1I2xC2yDwIbuNHN/1ynXdGgzHLqdCrXUPMwELJPc7yuU7XwJ8wCYUrq4aXX86HLnkMx7fPFvNeMk0uek9ma01magfEBIf+vJvZdWKiv48eUu9Cv31plAJsH6kSIsGc5TVYBYipkrQUNRxxJA15QT+uCURF96v3XuSS0k2Pf108AI=",
            note: "hard-unicode string"
        },
        ValidSec {
            sec1: "8f40e50a84a7462e2b8d24c28898ef1f23359fff50d8c509e6fb7ce06e142f9c",
            sec2: "b9b0a1e9cc20100c5faa3bbe2777303d25950616c4c6a3fa2e3e046f936ec2ba",
            shared: "ab99c122d4586cdd5c813058aa543d0e7233545dbf6874fc34a3d8d9a18fbbc3",
            salt: "b20989adc3ddc41cd2c435952c0d59a91315d8c5218d5040573fc3749543acaf",
            plaintext: "ability🤝的 ȺȾ",
            ciphertext: "ArIJia3D3cQc0sQ1lSwNWakTFdjFIY1QQFc/w3SVQ6yvPSc+7YCIFTmGk5OLuh1nhl6TvID7sGKLFUCWRW1eRfV/0a7sT46N3nTQzD7IE67zLWrYqGnE+0DDNz6sJ4hAaFrT",
            note: "",
        },
        ValidSec {
            sec1: "875adb475056aec0b4809bd2db9aa00cff53a649e7b59d8edcbf4e6330b0995c",
            sec2: "9c05781112d5b0a2a7148a222e50e0bd891d6b60c5483f03456e982185944aae",
            shared: "a449f2a85c6d3db0f44c64554a05d11a3c0988d645e4b4b2592072f63662f422",
            salt: "8d4442713eb9d4791175cb040d98d6fc5be8864d6ec2f89cf0895a2b2b72d1b1",
            plaintext: "pepper👀їжак",
            ciphertext: "Ao1EQnE+udR5EXXLBA2Y1vxb6IZNbsL4nPCJWisrctGx1TkkMfiHJxEeSdQ/4Rlaghn0okDCNYLihBsHrDzBsNRC27APmH9mmZcpcg66Mb0exH9V5/lLBWdQW+fcY9GpvXv0",
            note: "",
        },
        ValidSec {
            sec1: "eba1687cab6a3101bfc68fd70f214aa4cc059e9ec1b79fdb9ad0a0a4e259829f",
            sec2: "dff20d262bef9dfd94666548f556393085e6ea421c8af86e9d333fa8747e94b3",
            shared: "decde9938ffcb14fa7ff300105eb1bf239469af9baf376e69755b9070ae48c47",
            salt: "2180b52ae645fcf9f5080d81b1f0b5d6f2cd77ff3c986882bb549158462f3407",
            plaintext: "( ͡° ͜ʖ ͡°)",
            ciphertext: "AiGAtSrmRfz59QgNgbHwtdbyzXf/PJhogrtUkVhGLzQHiR8Hljs6Nl/XsNDAmCz6U1Z3NUGhbCtczc3wXXxDzFkjjMimxsf/74OEzu7LphUadM9iSWvVKPrNXY7lTD0B2muz",
            note: "",
        },
        ValidSec {
            sec1: "d5633530f5bcfebceb5584cfbbf718a30df0751b729dd9a789b9f30c0587d74e",
            sec2: "b74e6a341fb134127272b795a08b59250e5fa45a82a2eb4095e4ce9ed5f5e214",
            shared: "c6f2fde7aa00208c388f506455c31c3fa07caf8b516d43bf7514ee19edcda994",
            salt: "e4cd5f7ce4eea024bc71b17ad456a986a74ac426c2c62b0a15eb5c5c8f888b68",
            plaintext: "مُنَاقَشَةُ سُبُلِ اِسْتِخْدَامِ اللُّغَةِ فِي النُّظُمِ الْقَائِمَةِ وَفِيم يَخُصَّ التَّطْبِيقَاتُ الْحاسُوبِيَّةُ،",
            ciphertext: "AuTNX3zk7qAkvHGxetRWqYanSsQmwsYrChXrXFyPiItohfde4vHVRHUupr+Glh9JW4f9EY+w795hvRZbixs0EQgDZ7zwLlymVQI3NNvMqvemQzHUA1I5+9gSu8XSMwX9gDCUAjUJtntCkRt9+tjdy2Wa2ZrDYqCvgirvzbJTIC69Ve3YbKuiTQCKtVi0PA5ZLqVmnkHPIqfPqDOGj/a3dvJVzGSgeijcIpjuEgFF54uirrWvIWmTBDeTA+tlQzJHpB2wQnUndd2gLDb8+eKFUZPBifshD3WmgWxv8wRv6k3DeWuWEZQ70Z+YDpgpeOzuzHj0MDBwMAlY8Qq86Rx6pxY76PLDDfHh3rE2CHJEKl2MhDj7pGXao2o633vSRd9ueG8W",
            note: "",
        },
        ValidSec {
            sec1: "d5633530f5bcfebceb5584cfbbf718a30df0751b729dd9a789b9f30c0587d74e",
            sec2: "b74e6a341fb134127272b795a08b59250e5fa45a82a2eb4095e4ce9ed5f5e214",
            shared: "c6f2fde7aa00208c388f506455c31c3fa07caf8b516d43bf7514ee19edcda994",
            salt: "38d1ca0abef9e5f564e89761a86cee04574b6825d3ef2063b10ad75899e4b023",
            plaintext: "الكل في المجمو عة (5)",
            ciphertext: "AjjRygq++eX1ZOiXYahs7gRXS2gl0+8gY7EK11iZ5LAjTHmhdBC3meTY4A7Lv8s8B86MnmlUBJ8ebzwxFQzDyVCcdSbWFaKe0gigEBdXew7TjrjH8BCpAbtYjoa4YHa8GNjj7zH314ApVnwoByHdLHLB9Vr6VdzkxcJgA6oL4MAsRLg=",
            note: "",
        },
        ValidSec {
            sec1: "d5633530f5bcfebceb5584cfbbf718a30df0751b729dd9a789b9f30c0587d74e",
            sec2: "b74e6a341fb134127272b795a08b59250e5fa45a82a2eb4095e4ce9ed5f5e214",
            shared: "c6f2fde7aa00208c388f506455c31c3fa07caf8b516d43bf7514ee19edcda994",
            salt: "4f1a31909f3483a9e69c8549a55bbc9af25fa5bbecf7bd32d9896f83ef2e12e0",
            plaintext: "𝖑𝖆𝖟𝖞 社會科學院語學研究所",
            ciphertext: "Ak8aMZCfNIOp5pyFSaVbvJryX6W77Pe9MtmJb4PvLhLg/25Q5uBC88jl5ghtEREXX6o4QijPzM0uwmkeQ54/6aIqUyzGNVdryWKZ0mee2lmVVWhU+26X6XGFQ5DGRn+1v0POsFUCZ/REh35+beBNHnyvjxD/rbrMfhP2Blc8X5m8Xvk=",
            note: "",
        },
        ValidSec {
            sec1: "d5633530f5bcfebceb5584cfbbf718a30df0751b729dd9a789b9f30c0587d74e",
            sec2: "b74e6a341fb134127272b795a08b59250e5fa45a82a2eb4095e4ce9ed5f5e214",
            shared: "c6f2fde7aa00208c388f506455c31c3fa07caf8b516d43bf7514ee19edcda994",
            salt: "a3e219242d85465e70adcd640b564b3feff57d2ef8745d5e7a0663b2dccceb54",
            plaintext: "🙈 🙉 🙊 0️⃣ 1️⃣ 2️⃣ 3️⃣ 4️⃣ 5️⃣ 6️⃣ 7️⃣ 8️⃣ 9️⃣ 🔟 Powerلُلُصّبُلُلصّبُررً ॣ ॣh ॣ ॣ冗",
            ciphertext: "AqPiGSQthUZecK3NZAtWSz/v9X0u+HRdXnoGY7LczOtU9bUC2ji2A2udRI2VCEQZ7IAmYRRgxodBtd5Yi/5htCUczf1jLHxIt9AhVAZLKuRgbWOuEMq5RBybkxPsSeAkxzXVOlWHZ1Febq5ogkjqY/6Xj8CwwmaZxfbx+d1BKKO3Wa+IFuXwuVAZa1Xo+fan+skyf+2R5QSj10QGAnGO7odAu/iZ9A28eMoSNeXsdxqy1+PRt5Zk4i019xmf7C4PDGSzgFZSvQ2EzusJN5WcsnRFmF1L5rXpX1AYo8HusOpWcGf9PjmFbO+8spUkX1W/T21GRm4o7dro1Y6ycgGOA9BsiQ==",
            note: "emoji and lang 7"
        },
    ];

    const VALID_PUB: [ValidPub; 3] = [
        ValidPub {
            sec1: "fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364139",
            pub2: "0000000000000000000000000000000000000000000000000000000000000002",
            shared: "7a1ccf5ce5a08e380f590de0c02776623b85a61ae67cfb6a017317e505b7cb51",
            salt: "a000000000000000000000000000000000000000000000000000000000000001",
            plaintext: "⁰⁴⁵₀₁₂",
            ciphertext: "AqAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAB2+xmGnjIMPMqqJGmjdYAYZUDUyEEUO3/evHUaO40LePeR91VlMVZ7I+nKJPkaUiKZ3cQiQnA86Uwti2IxepmzOFN",
            note: "sec1 = n-2, pub2: random, 0x02"
        },
        ValidPub {
            sec1: "0000000000000000000000000000000000000000000000000000000000000002",
            pub2: "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdeb",
            shared: "aa971537d741089885a0b48f2730a125e15b36033d089d4537a4e1204e76b39e",
            salt: "b000000000000000000000000000000000000000000000000000000000000002",
            plaintext: "A Peer-to-Peer Electronic Cash System",
            ciphertext: "ArAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAACyuqG6RycuPyDPtwxzTcuMQu+is3N5XuWTlvCjligVaVBRydexaylXbsX592MEd3/Jt13BNL/GlpYpGDvLS4Tt/+2s9FX/16e/RDc+czdwXglc4DdSHiq+O06BvvXYfEQOPw=",
            note: "sec1 = 2, pub2: "
        },
        ValidPub {
            sec1: "0000000000000000000000000000000000000000000000000000000000000001",
            pub2: "79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798",
            shared: "79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798",
            salt: "79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798",
            plaintext: "A purely peer-to-peer version of electronic cash would allow online payments to be sent directly from one party to another without going through a financial institution. Digital signatures provide part of the solution, but the main benefits are lost if a trusted third party is still required to prevent double-spending.",
            ciphertext: "Anm+Zn753LusVaBilc6HCwcCm/zbLc4o2VnygVsW+BeYb9wHyKevpe7ohJ6OkpceFcb0pySY8TLGwT7Q3zWNDKxc9blXanxKborEXkQH8xNaB2ViJfgxpkutbwbYd0Grix34xzaZBASufdsNm7R768t51tI6sdS0nms6kWLVJpEGu6Ke4Bldv4StJtWBLaTcgsgN+4WxDbBhC/nhwjEQiBBbbmUrPWjaVZXjl8dzzPrYtkSoeBNJs/UNvDwym4+qrmhv4ASTvVflpZgLlSe4seqeu6dWoRqn8uRHZQnPs+XhqwbdCHpeKGB3AfGBykZY0RIr0tjarWdXNasGbIhGM3GiLasioJeabAZw0plCevDkKpZYDaNfMJdzqFVJ8UXRIpvDpQad0SOm8lLum/aBzUpLqTjr3RvSlhYdbuODpd9pR5K60k4L2N8nrPtBv08wlilQg2ymwQgKVE6ipxIzzKMetn8+f0nQ9bHjWFJqxetSuMzzArTUQl9c4q/DwZmCBhI2",
            note: "sec1 == pub2 == salt"
        }
    ];

    const INVALID_PUB: [InvalidPub; 7] = [
        InvalidPub {
            sec1: "11063318c5cb3cd9cafcced42b4db5ea02ec976ed995962d2bc1fa1e9b52e29f",
            pub2: "5c49873b6eac3dd363325250cc55d5dd4c7ce9a885134580405736d83506bb74",
            shared: "e2aad10de00913088e5cb0f73fa526a6a17e95763cc5b2a127022f5ea5a73445",
            salt: "ad408d4be8616dc84bb0bf046454a2a102edac937c35209c43cd7964c5feb781",
            plaintext: "⚠️",
            ciphertext: "AK1AjUvoYW3IS7C/BGRUoqEC7ayTfDUgnEPNeWTF/reBA4fZmoHrtrz5I5pCHuwWZ22qqL/Xt1VidEZGMLds0yaJ5VwUbeEifEJlPICOFt1ssZJxCUf43HvRwCVTFskbhSMh",
            note: "unknown encryption version: 0",
            error: Error::UnknownVersion(0x00),
        },
        InvalidPub {
            sec1: "2573d1e9b9ac5de5d570f652cbb9e8d4f235e3d3d334181448e87c417f374e83",
            pub2: "8348c2d35549098706e5bab7966d9a9c72fbf6554e918f41c2b6cb275f79ec13",
            shared: "8673ec68393a997bfad7eab8661461daf8b3931b7e885d78312a3fb7fe17f41a",
            salt: "daaea5ca345b268e5b62060ca72c870c48f713bc1e00ff3fc0ddb78e826f10db",
            plaintext: "n o s t r",
            ciphertext: "Atqupco0WyaOW2IGDKcshwxI9xO8HgD/P8Ddt46CbxDbOsrsqIEybscEwg5rnI/Cx03mDSmeweOLKD,7dw5BDZQDxXSlCwX1LIcTJEZaJPTz98Ftu0zSE0d93ED7OtdlvNeZx",
            note: "invalid base64",
            error: Error::Base64Decode(base64::DecodeError::InvalidLength),
        },
        InvalidPub {
            sec1: "5a2f39347fed3883c9fe05868a8f6156a292c45f606bc610495fcc020ed158f7",
            pub2: "775bbfeba58d07f9d1fbb862e306ac780f39e5418043dadb547c7b5900245e71",
            shared: "2e70c0a1cde884b88392458ca86148d859b273a5695ede5bbe41f731d7d88ffd",
            salt: "09ff97750b084012e15ecb84614ce88180d7b8ec0d468508a86b6d70c0361a25",
            plaintext: "¯\\_(ツ)_/¯",
            ciphertext: "Agn/l3ULCEAS4V7LhGFM6IGA17jsDUaFCKhrbXDANholdUejFZPARM22IvOqp1U/UmFSkeSyTBYbbwy5ykmi+mKiEcWL+nVmTOf28MMiC+rTpZys/8p1hqQFpn+XWZRPrVay",
            note: "invalid MAC",
            error: Error::V2(ErrorV2::InvalidHmac),
        },
        InvalidPub {
            sec1: "067eda13c4a36090ad28a7a183e9df611186ca01f63cb30fcdfa615ebfd6fb6d",
            pub2: "32c1ece2c5dd2160ad03b243f50eff12db605b86ac92da47eacc78144bf0cdd3",
            shared: "a808915e31afc5b853d654d2519632dac7298ee2ecddc11695b8eba925935c2a",
            salt: "65b14b0b949aaa7d52c417eb753b390e8ad6d84b23af4bec6d9bfa3e03a08af4",
            plaintext: "🥎",
            ciphertext: "AmWxSwuUmqp9UsQX63U7OQ6K1thLI69L7G2b+j4DoIr0U0P/M1/oKm95z8qz6Kg0zQawLzwk3DskvWA2drXP4zK+tzHpKvWq0KOdx5MdypboSQsP4NXfhh2KoUffjkyIOiMA",
            note: "invalid MAC",
            error: Error::V2(ErrorV2::InvalidHmac),
        },
        InvalidPub {
            sec1: "3e7be560fb9f8c965c48953dbd00411d48577e200cf00d7cc427e49d0e8d9c01",
            pub2: "e539e5fee58a337307e2a937ee9a7561b45876fb5df405c5e7be3ee564b239cc",
            shared: "6ee3efc4255e3b8270e5dd3f7dc7f6b60878cda6218c8df34a3261cd48744931",
            salt: "7ab65dbb8bbc2b8e35cafb5745314e1f050325a864d11d0475ef75b3660d91c1",
            plaintext: "elliptic-curve cryptography",
            ciphertext: "Anq2XbuLvCuONcr7V0UxTh8FAyWoZNEdBHXvdbNmDZHBu7F9m36yBd58mVUBB5ktBTOJREDaQT1KAyPmZidP+IRea1lNw5YAEK7+pbnpfCw8CD0i2n8Pf2IDWlKDhLiVvatw",
            note: "invalid padding",
            error: Error::V2(ErrorV2::MessageEmpty),
        },
        InvalidPub {
            sec1: "c22e1d4de967aa39dc143354d8f596cec1d7c912c3140831fff2976ce3e387c1",
            pub2: "4e405be192677a2da95ffc733950777213bf880cf7c3b084eeb6f3fe5bd43705",
            shared: "1675a773dbf6fbcbef6a293004a4504b6c856978be738b10584b0269d437c8d1",
            salt: "7d4283e3b54c885d6afee881f48e62f0a3f5d7a9e1cb71ccab594a7882c39330",
            plaintext: "Peer-to-Peer",
            ciphertext: "An1Cg+O1TIhdav7ogfSOYvCj9dep4ctxzKtZSniCw5MwhT0hvSnF9Xjp9Lml792qtNbmAVvR6laukTe9eYEjeWPpZFxtkVpYTbbL9wDKFeplDMKsUKVa+roSeSvv0ela9seDVl2Sfso=",
            note: "invalid padding",
            error: Error::V2(ErrorV2::InvalidPadding),
        },
        InvalidPub {
            sec1: "be1edab14c5912e5c59084f197f0945242e969c363096cccb59af8898815096f",
            pub2: "9eaf0775d971e4941c97189232542e1daefcdb7dddafc39bcea2520217710ba2",
            shared: "1741a44c052d5ae363c7845441f73d2b6c28d9bfb3006190012bba12eb4c774b",
            salt: "6f9fd72667c273acd23ca6653711a708434474dd9eb15c3edb01ce9a95743e9b",
            plaintext: "censorship-resistant and global social network",
            ciphertext: "Am+f1yZnwnOs0jymZTcRpwhDRHTdnrFcPtsBzpqVdD6bL9HUMo3Mjkz4bjQo/FJF2LWHmaCr9Byc3hU9D7we+EkNBWenBHasT1G52fZk9r3NKeOC1hLezNwBLr7XXiULh+NbMBDtJh9/aQh1uZ9EpAfeISOzbZXwYwf0P5M85g9XER8hZ2fgJDLb4qMOuQRG6CrPezhr357nS3UHwPC2qHo3uKACxhE+2td+965yDcvMTx4KYTQg1zNhd7PA5v/WPnWeq2B623yLxlevUuo/OvXplFho3QVy7s5QZVop6qV2g2/l/SIsvD0HIcv3V35sywOCBR0K4VHgduFqkx/LEF3NGgAbjONXQHX8ZKushsEeR4TxlFoRSovAyYjhWolz+Ok3KJL2Ertds3H+M/Bdl2WnZGT0IbjZjn3DS+b1Ke0R0X4Onww2ZG3+7o6ncIwTc+lh1O7YQn00V0HJ+EIp03heKV2zWdVSC615By/+Yt9KAiV56n5+02GAuNqA",
            note: "invalid padding",
            error: Error::V2(ErrorV2::InvalidPadding),
        }
    ];

    const INVALID_KEYS: [InvalidKeys; 5] = [
        InvalidKeys {
            sec1: "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
            pub2: "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
            note: "sec1 higher than curve.n",
        },
        InvalidKeys {
            sec1: "0000000000000000000000000000000000000000000000000000000000000000",
            pub2: "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
            note: "sec1 is 0",
        },
        InvalidKeys {
            sec1: "fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364139",
            pub2: "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
            note: "pub2 is invalid, no sqrt, all-ff",
        },
        InvalidKeys {
            sec1: "fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141",
            pub2: "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
            note: "sec1 == curve.n",
        },
        InvalidKeys {
            sec1: "0000000000000000000000000000000000000000000000000000000000000002",
            pub2: "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
            note: "pub2 is invalid, no sqrt",
        },
    ];

    const PADDING: [(usize, usize); 24] = [
        (16, 32),
        (32, 32),
        (33, 64),
        (37, 64),
        (45, 64),
        (49, 64),
        (64, 64),
        (65, 96),
        (100, 128),
        (111, 128),
        (200, 224),
        (250, 256),
        (320, 320),
        (383, 384),
        (384, 384),
        (400, 448),
        (500, 512),
        (512, 512),
        (515, 640),
        (700, 768),
        (800, 896),
        (900, 1024),
        (1020, 1024),
        (74123, 81920),
    ];

    #[test]
    pub fn test_valid_sec_test_vectors() {
        let secp = Secp256k1::new();

        for vec in VALID_SEC.into_iter() {
            let vector = vec.parsed();

            // Test shared key
            let shared_key =
                util::generate_shared_key(&vector.sec1, &vector.sec2.x_only_public_key(&secp).0);
            assert_eq!(
                shared_key, vector.shared,
                "Conversation key failure on {}",
                vector.note
            );

            // Test decryption
            let plaintext = nip44::decrypt(
                &vector.sec1,
                &vector.sec2.x_only_public_key(&secp).0,
                vector.ciphertext,
            )
            .unwrap();
            assert_eq!(
                plaintext, vector.plaintext,
                "Decryption does not match on {}",
                vector.note
            );
        }
    }

    #[test]
    pub fn test_valid_pub_test_vectors() {
        for vec in VALID_PUB.into_iter() {
            let vector = vec.parsed();

            // Test conversation key
            let shared_key = util::generate_shared_key(&vector.sec1, &vector.pub2);
            assert_eq!(
                shared_key, vector.shared,
                "Conversation key failure on {}",
                vector.note
            );

            // Test decryption
            let plaintext = nip44::decrypt(&vector.sec1, &vector.pub2, &vector.ciphertext).unwrap();
            assert_eq!(
                plaintext, vector.plaintext,
                "Decryption does not match on {}",
                vector.note
            );
        }
    }

    #[test]
    pub fn test_invalid_pub_test_vectors() {
        for vec in INVALID_PUB.into_iter() {
            let vector = vec.parsed();

            // Test shared key
            let shared_key = util::generate_shared_key(&vector.sec1, &vector.pub2);
            assert_eq!(
                shared_key, vector.shared,
                "Shared key failure on {}",
                vector.note
            );

            // Test decryption fails
            let result = nip44::decrypt(&vector.sec1, &vector.pub2, &vector.ciphertext);
            assert!(
                result.is_err(),
                "Should not have decrypted: {}",
                vector.note
            );
            let err = result.unwrap_err();
            assert_eq!(err, vector.error, "Plaintext was {}", vector.plaintext);
        }
    }

    #[test]
    pub fn test_invalid_keys() {
        for vector in INVALID_KEYS.into_iter() {
            let sec1 = SecretKey::from_str(vector.sec1);
            let pub2 = XOnlyPublicKey::from_str(vector.pub2);
            assert!(
                sec1.is_err() || pub2.is_err(),
                "One of the keys should have failed: {}",
                vector.note
            );
        }
    }

    #[test]
    fn test_padding_length() {
        for (len, pad) in PADDING.into_iter() {
            assert_eq!(calc_padding(len), pad);
        }
    }
}
