// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP57
//!
//! <https://github.com/nostr-protocol/nips/blob/master/57.md>

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt;

use aes::cipher::block_padding::Pkcs7;
use aes::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use aes::Aes256;
use bitcoin::bech32::{self, FromBase32, ToBase32, Variant};
use bitcoin::hashes::sha256::Hash as Sha256Hash;
use bitcoin::hashes::Hash;
#[cfg(feature = "std")]
use bitcoin::secp256k1::rand::rngs::OsRng;
use bitcoin::secp256k1::rand::{CryptoRng, RngCore};
use bitcoin::secp256k1::{self, Secp256k1, SecretKey, Signing, XOnlyPublicKey};
use cbc::{Decryptor, Encryptor};

use super::nip01::Coordinate;
use crate::event::builder::Error as BuilderError;
use crate::key::Error as KeyError;
#[cfg(feature = "std")]
use crate::types::time::Instant;
use crate::types::time::TimeSupplier;
#[cfg(feature = "std")]
use crate::SECP256K1;
use crate::{
    event, util, Event, EventBuilder, EventId, JsonUtil, Keys, Kind, Tag, Timestamp, UncheckedUrl,
};

type Aes256CbcEnc = Encryptor<Aes256>;
type Aes256CbcDec = Decryptor<Aes256>;

const PRIVATE_ZAP_MSG_BECH32_PREFIX: &str = "pzap";
const PRIVATE_ZAP_IV_BECH32_PREFIX: &str = "iv";

#[allow(missing_docs)]
#[derive(Debug)]
pub enum Error {
    Key(KeyError),
    Builder(BuilderError),
    Event(event::Error),
    Bech32(bech32::Error),
    Secp256k1(secp256k1::Error),
    InvalidPrivateZapMessage,
    PrivateZapMessageNotFound,
    /// Wrong prefix or variant
    WrongBech32PrefixOrVariant,
    /// Wrong encryption block mode
    WrongBlockMode,
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Key(e) => write!(f, "{e}"),
            Self::Builder(e) => write!(f, "{e}"),
            Self::Event(e) => write!(f, "{e}"),
            Self::Bech32(e) => write!(f, "{e}"),
            Self::Secp256k1(e) => write!(f, "{e}"),
            Self::InvalidPrivateZapMessage => write!(f, "Invalid private zap message"),
            Self::PrivateZapMessageNotFound => write!(f, "Private zap message not found"),
            Self::WrongBech32PrefixOrVariant => write!(f, "Wrong bech32 prefix or variant"),
            Self::WrongBlockMode => write!(
                f,
                "Wrong encryption block mode. The content must be encrypted using CBC mode!"
            ),
        }
    }
}

impl From<KeyError> for Error {
    fn from(e: KeyError) -> Self {
        Self::Key(e)
    }
}

impl From<BuilderError> for Error {
    fn from(e: BuilderError) -> Self {
        Self::Builder(e)
    }
}

impl From<event::Error> for Error {
    fn from(e: event::Error) -> Self {
        Self::Event(e)
    }
}

impl From<bech32::Error> for Error {
    fn from(e: bech32::Error) -> Self {
        Self::Bech32(e)
    }
}

impl From<secp256k1::Error> for Error {
    fn from(e: secp256k1::Error) -> Self {
        Self::Secp256k1(e)
    }
}

/* /// Zap Type
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ZapType {
    /// Public
    Public,
    /// Private
    Private,
    /// Anonymous
    Anonymous,
} */

/// Zap Request Data
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ZapRequestData {
    /// Public key of the recipient
    pub public_key: XOnlyPublicKey,
    /// List of relays the recipient's wallet should publish its zap receipt to
    pub relays: Vec<UncheckedUrl>,
    /// Message
    pub message: String,
    /// Amount in `millisats` the sender intends to pay
    pub amount: Option<u64>,
    /// Lnurl pay url of the recipient, encoded using bech32 with the prefix lnurl.
    pub lnurl: Option<String>,
    /// Event ID
    pub event_id: Option<EventId>,
    /// NIP-33 event coordinate that allows tipping parameterized replaceable events such as NIP-23 long-form notes.
    pub event_coordinate: Option<Coordinate>,
}

impl ZapRequestData {
    /// New Zap Request Data
    pub fn new<I>(public_key: XOnlyPublicKey, relays: I) -> Self
    where
        I: IntoIterator<Item = UncheckedUrl>,
    {
        Self {
            public_key,
            relays: relays.into_iter().collect(),
            message: String::new(),
            amount: None,
            lnurl: None,
            event_id: None,
            event_coordinate: None,
        }
    }

    /// Message
    pub fn message<S>(self, message: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            message: message.into(),
            ..self
        }
    }

    /// Amount in `millisats` the sender intends to pay
    pub fn amount(self, amount: u64) -> Self {
        Self {
            amount: Some(amount),
            ..self
        }
    }

    /// Lnurl pay url of the recipient, encoded using bech32 with the prefix lnurl.
    pub fn lnurl<S>(self, lnurl: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            lnurl: Some(lnurl.into()),
            ..self
        }
    }

    /// Event ID
    pub fn event_id(self, event_id: EventId) -> Self {
        Self {
            event_id: Some(event_id),
            ..self
        }
    }

    /// NIP-33 event coordinate that allows tipping parameterized replaceable events such as NIP-23 long-form notes.
    pub fn event_coordinate(self, event_coordinate: Coordinate) -> Self {
        Self {
            event_coordinate: Some(event_coordinate),
            ..self
        }
    }
}

impl From<ZapRequestData> for Vec<Tag> {
    fn from(data: ZapRequestData) -> Self {
        let ZapRequestData {
            public_key,
            relays,
            amount,
            lnurl,
            event_id,
            event_coordinate,
            ..
        } = data;

        let mut tags: Vec<Tag> = vec![Tag::public_key(public_key)];

        if !relays.is_empty() {
            tags.push(Tag::Relays(relays));
        }

        if let Some(event_id) = event_id {
            tags.push(Tag::event(event_id));
        }

        if let Some(event_coordinate) = event_coordinate {
            tags.push(event_coordinate.into());
        }

        if let Some(amount) = amount {
            tags.push(Tag::Amount {
                millisats: amount,
                bolt11: None,
            });
        }

        if let Some(lnurl) = lnurl {
            tags.push(Tag::Lnurl(lnurl));
        }

        tags
    }
}

/// Create **anonymous** zap request
#[cfg(feature = "std")]
pub fn anonymous_zap_request(data: ZapRequestData) -> Result<Event, Error> {
    let keys = Keys::generate();
    let message: String = data.message.clone();
    let mut tags: Vec<Tag> = data.into();
    tags.push(Tag::Anon { msg: None });
    Ok(EventBuilder::new(Kind::ZapRequest, message, tags).to_event(&keys)?)
}

/// Create **private** zap request
#[cfg(feature = "std")]
pub fn private_zap_request(data: ZapRequestData, keys: &Keys) -> Result<Event, Error> {
    private_zap_request_with_ctx(&SECP256K1, &mut OsRng, &Instant::now(), data, keys)
}

/// Create **private** zap request
pub fn private_zap_request_with_ctx<C, R, T>(
    secp: &Secp256k1<C>,
    rng: &mut R,
    supplier: &T,
    data: ZapRequestData,
    keys: &Keys,
) -> Result<Event, Error>
where
    C: Signing,
    R: RngCore + CryptoRng,
    T: TimeSupplier,
{
    let created_at: Timestamp = Timestamp::now_with_supplier(supplier);

    // Create encryption key
    let secret_key: SecretKey =
        create_encryption_key(&keys.secret_key()?, &data.public_key, created_at)?;

    // Compose encrypted message
    let mut tags: Vec<Tag> = vec![Tag::public_key(data.public_key)];
    if let Some(event_id) = data.event_id {
        tags.push(Tag::event(event_id));
    }
    let msg: String = EventBuilder::new(Kind::ZapPrivateMessage, &data.message, tags)
        .to_event_with_ctx(secp, rng, supplier, keys)?
        .as_json();
    let msg: String = encrypt_private_zap_message(rng, &secret_key, &data.public_key, msg)?;

    // Compose event
    let mut tags: Vec<Tag> = data.into();
    tags.push(Tag::Anon { msg: Some(msg) });
    let private_zap_keys: Keys = Keys::new_with_ctx(secp, secret_key);
    Ok(EventBuilder::new(Kind::ZapRequest, "", tags)
        .custom_created_at(created_at)
        .to_event_with_ctx(secp, rng, supplier, &private_zap_keys)?)
}

/// Create NIP57 encryption key for **private** zap
pub fn create_encryption_key(
    secret_key: &SecretKey,
    public_key: &XOnlyPublicKey,
    created_at: Timestamp,
) -> Result<SecretKey, Error> {
    let mut unhashed: String = secret_key.display_secret().to_string();
    unhashed.push_str(&public_key.to_string());
    unhashed.push_str(&created_at.to_string());
    let hash = Sha256Hash::hash(unhashed.as_bytes());
    Ok(SecretKey::from_slice(hash.as_byte_array())?)
}

fn encrypt_private_zap_message<R, T>(
    rng: &mut R,
    secret_key: &SecretKey,
    public_key: &XOnlyPublicKey,
    msg: T,
) -> Result<String, Error>
where
    R: RngCore,
    T: AsRef<[u8]>,
{
    let key: [u8; 32] = util::generate_shared_key(secret_key, public_key);
    let mut iv: [u8; 16] = [0u8; 16];
    rng.fill_bytes(&mut iv);

    let cipher = Aes256CbcEnc::new(&key.into(), &iv.into());
    let result: Vec<u8> = cipher.encrypt_padded_vec_mut::<Pkcs7>(msg.as_ref());

    // Bech32 msg
    let data = result.to_base32();
    let encrypted_bech32_msg =
        bech32::encode(PRIVATE_ZAP_MSG_BECH32_PREFIX, data, Variant::Bech32)?;

    // Bech32 IV
    let data = iv.to_base32();
    let iv_bech32 = bech32::encode(PRIVATE_ZAP_IV_BECH32_PREFIX, data, Variant::Bech32)?;

    Ok(format!("{encrypted_bech32_msg}_{iv_bech32}"))
}

fn extract_anon_tag_message(event: &Event) -> Result<&String, Error> {
    for tag in event.tags.iter() {
        if let Tag::Anon { msg } = tag {
            return msg.as_ref().ok_or(Error::InvalidPrivateZapMessage);
        }
    }
    Err(Error::PrivateZapMessageNotFound)
}

/// Decrypt **private** zap message
pub fn decrypt_private_zap_message(
    secret_key: &SecretKey,
    public_key: &XOnlyPublicKey,
    private_zap_event: &Event,
) -> Result<Event, Error> {
    let secret_key: SecretKey =
        create_encryption_key(secret_key, public_key, private_zap_event.created_at)?;
    let key: [u8; 32] = util::generate_shared_key(&secret_key, public_key);

    let msg: &String = extract_anon_tag_message(private_zap_event)?;
    let mut splitted = msg.split('_');

    let msg: &str = splitted.next().ok_or(Error::InvalidPrivateZapMessage)?;
    let iv: &str = splitted.next().ok_or(Error::InvalidPrivateZapMessage)?;

    // IV
    let (hrp, data, checksum) = bech32::decode(iv)?;
    if hrp != PRIVATE_ZAP_IV_BECH32_PREFIX || checksum != Variant::Bech32 {
        return Err(Error::WrongBech32PrefixOrVariant);
    }
    let iv: Vec<u8> = Vec::from_base32(&data)?;

    // Msg
    let (hrp, data, checksum) = bech32::decode(msg)?;
    if hrp != PRIVATE_ZAP_MSG_BECH32_PREFIX || checksum != Variant::Bech32 {
        return Err(Error::WrongBech32PrefixOrVariant);
    }
    let msg: Vec<u8> = Vec::from_base32(&data)?;

    // Decrypt
    let cipher = Aes256CbcDec::new(&key.into(), iv.as_slice().into());
    let result: Vec<u8> = cipher
        .decrypt_padded_vec_mut::<Pkcs7>(&msg)
        .map_err(|_| Error::WrongBlockMode)?;

    // TODO: check if event kind is equal to 9733
    Ok(Event::from_json(result)?)
}

#[cfg(feature = "std")]
#[cfg(test)]
mod tests {
    use core::str::FromStr;

    use super::*;
    use crate::FromBech32;

    #[test]
    fn test_encrypt_decrypt_private_zap_message() {
        let secret_key =
            SecretKey::from_str("6b911fd37cdf5c81d4c0adb1ab7fa822ed253ab0ad9aa18d77257c88b29b718e")
                .unwrap();
        let alice_keys = Keys::new(secret_key);

        let public_key = XOnlyPublicKey::from_bech32(
            "npub14f8usejl26twx0dhuxjh9cas7keav9vr0v8nvtwtrjqx3vycc76qqh9nsy",
        )
        .unwrap();
        let relays = [UncheckedUrl::from("wss://relay.damus.io")];
        let msg = "Private Zap message!";
        let data = ZapRequestData::new(public_key, relays).message(msg);
        let private_zap = private_zap_request(data, &alice_keys).unwrap();

        let private_zap_msg =
            decrypt_private_zap_message(&secret_key, &public_key, &private_zap).unwrap();

        assert_eq!(msg, &private_zap_msg.content,)
    }
}
