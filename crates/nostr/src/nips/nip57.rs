// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP57: Lightning Zaps
//!
//! <https://github.com/nostr-protocol/nips/blob/master/57.md>

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt;

use aes::cipher::block_padding::Pkcs7;
use aes::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use aes::Aes256;
use bech32::{Bech32, Hrp};
use bitcoin::hashes::sha256::Hash as Sha256Hash;
use bitcoin::hashes::Hash;
#[cfg(feature = "std")]
use bitcoin::secp256k1::rand::rngs::OsRng;
use bitcoin::secp256k1::rand::{CryptoRng, RngCore};
use bitcoin::secp256k1::{self, Secp256k1, Signing, Verification};
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
    event, util, Event, EventBuilder, EventId, JsonUtil, Keys, Kind, PublicKey, SecretKey, Tag,
    TagStandard, Timestamp, Url,
};

type Aes256CbcEnc = Encryptor<Aes256>;
type Aes256CbcDec = Decryptor<Aes256>;

const PRIVATE_ZAP_MSG_BECH32_PREFIX: Hrp = Hrp::parse_unchecked("pzap");
const PRIVATE_ZAP_IV_BECH32_PREFIX: Hrp = Hrp::parse_unchecked("iv");

#[allow(missing_docs)]
#[derive(Debug)]
pub enum Error {
    Fmt(fmt::Error),
    Key(KeyError),
    Builder(BuilderError),
    Event(event::Error),
    Bech32Decode(bech32::DecodeError),
    Bech32Encode(bech32::EncodeError),
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
            Self::Fmt(e) => write!(f, "{e}"),
            Self::Key(e) => write!(f, "{e}"),
            Self::Builder(e) => write!(f, "{e}"),
            Self::Event(e) => write!(f, "{e}"),
            Self::Bech32Decode(e) => write!(f, "{e}"),
            Self::Bech32Encode(e) => write!(f, "{e}"),
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

impl From<fmt::Error> for Error {
    fn from(e: fmt::Error) -> Self {
        Self::Fmt(e)
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

impl From<bech32::DecodeError> for Error {
    fn from(e: bech32::DecodeError) -> Self {
        Self::Bech32Decode(e)
    }
}

impl From<bech32::EncodeError> for Error {
    fn from(e: bech32::EncodeError) -> Self {
        Self::Bech32Encode(e)
    }
}

impl From<secp256k1::Error> for Error {
    fn from(e: secp256k1::Error) -> Self {
        Self::Secp256k1(e)
    }
}

/// Zap Type
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ZapType {
    /// Public
    Public,
    /// Private
    Private,
    /// Anonymous
    Anonymous,
}

/// Zap Request Data
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ZapRequestData {
    /// Public key of the recipient
    pub public_key: PublicKey,
    /// List of relays the recipient's wallet should publish its zap receipt to
    pub relays: Vec<Url>,
    /// Message
    pub message: String,
    /// Amount in `millisats` the sender intends to pay
    pub amount: Option<u64>,
    /// Lnurl pay url of the recipient, encoded using bech32 with the prefix lnurl.
    pub lnurl: Option<String>,
    /// Event ID
    pub event_id: Option<EventId>,
    /// NIP33 event coordinate that allows tipping parameterized replaceable events such as NIP23 long-form notes.
    pub event_coordinate: Option<Coordinate>,
}

impl ZapRequestData {
    /// New Zap Request Data
    pub fn new<I>(public_key: PublicKey, relays: I) -> Self
    where
        I: IntoIterator<Item = Url>,
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

    /// NIP33 event coordinate that allows tipping parameterized replaceable events such as NIP23 long-form notes.
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
            tags.push(Tag::from_standardized_without_cell(TagStandard::Relays(
                relays,
            )));
        }

        if let Some(event_id) = event_id {
            tags.push(Tag::event(event_id));
        }

        if let Some(event_coordinate) = event_coordinate {
            tags.push(event_coordinate.into());
        }

        if let Some(amount) = amount {
            tags.push(Tag::from_standardized_without_cell(TagStandard::Amount {
                millisats: amount,
                bolt11: None,
            }));
        }

        if let Some(lnurl) = lnurl {
            tags.push(Tag::from_standardized_without_cell(TagStandard::Lnurl(
                lnurl,
            )));
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
    tags.push(Tag::from_standardized_without_cell(TagStandard::Anon {
        msg: None,
    }));
    Ok(EventBuilder::new(Kind::ZapRequest, message)
        .tags(tags)
        .sign_with_keys(&keys)?)
}

/// Create **private** zap request
#[inline]
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
    C: Signing + Verification,
    R: RngCore + CryptoRng,
    T: TimeSupplier,
{
    let created_at: Timestamp = Timestamp::now_with_supplier(supplier);

    // Create encryption key
    let secret_key: SecretKey =
        create_encryption_key(keys.secret_key(), &data.public_key, created_at)?;

    // Compose encrypted message
    let mut tags: Vec<Tag> = vec![Tag::public_key(data.public_key)];
    if let Some(event_id) = data.event_id {
        tags.push(Tag::event(event_id));
    }
    let msg: String = EventBuilder::new(Kind::ZapPrivateMessage, &data.message)
        .tags(tags)
        .sign_with_ctx(secp, rng, supplier, keys)?
        .as_json();
    let msg: String = encrypt_private_zap_message(rng, &secret_key, &data.public_key, msg)?;

    // Compose event
    let mut tags: Vec<Tag> = data.into();
    tags.push(Tag::from_standardized_without_cell(TagStandard::Anon {
        msg: Some(msg),
    }));
    let private_zap_keys: Keys = Keys::new_with_ctx(secp, secret_key);
    Ok(EventBuilder::new(Kind::ZapRequest, "")
        .tags(tags)
        .custom_created_at(created_at)
        .sign_with_ctx(secp, rng, supplier, &private_zap_keys)?)
}

/// Create NIP57 encryption key for **private** zap
pub fn create_encryption_key(
    secret_key: &SecretKey,
    public_key: &PublicKey,
    created_at: Timestamp,
) -> Result<SecretKey, Error> {
    let mut unhashed: String = secret_key.to_secret_hex();
    unhashed.push_str(&public_key.to_string());
    unhashed.push_str(&created_at.to_string());
    let hash = Sha256Hash::hash(unhashed.as_bytes());
    Ok(SecretKey::from_slice(hash.as_byte_array())?)
}

/// Encrypt a private zap message using the given keys
pub fn encrypt_private_zap_message<R, T>(
    rng: &mut R,
    secret_key: &SecretKey,
    public_key: &PublicKey,
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
    let msg: Vec<u8> = cipher.encrypt_padded_vec_mut::<Pkcs7>(msg.as_ref());

    // Bech32 msg
    let encrypted_bech32_msg: String =
        bech32::encode::<Bech32>(PRIVATE_ZAP_MSG_BECH32_PREFIX, &msg)?;

    // Bech32 IV
    let iv_bech32: String = bech32::encode::<Bech32>(PRIVATE_ZAP_IV_BECH32_PREFIX, &iv)?;

    Ok(format!("{encrypted_bech32_msg}_{iv_bech32}"))
}

fn extract_anon_tag_message(event: &Event) -> Result<&String, Error> {
    for tag in event.tags.iter() {
        if let Some(TagStandard::Anon { msg }) = tag.as_standardized() {
            return msg.as_ref().ok_or(Error::InvalidPrivateZapMessage);
        }
    }
    Err(Error::PrivateZapMessageNotFound)
}

/// Decrypt **private** zap message that was sent by the owner of the secret key
pub fn decrypt_sent_private_zap_message(
    secret_key: &SecretKey,
    public_key: &PublicKey,
    private_zap_event: &Event,
) -> Result<Event, Error> {
    // Re-create our ephemeral encryption key
    let secret_key: SecretKey =
        create_encryption_key(secret_key, public_key, private_zap_event.created_at)?;
    let key: [u8; 32] = util::generate_shared_key(&secret_key, public_key);

    // decrypt like normal
    decrypt_private_zap_message(key, private_zap_event)
}

/// Decrypt **private** zap message that was received by the owner of the secret key
#[inline]
pub fn decrypt_received_private_zap_message(
    secret_key: &SecretKey,
    private_zap_event: &Event,
) -> Result<Event, Error> {
    let key: [u8; 32] = util::generate_shared_key(secret_key, &private_zap_event.pubkey);
    decrypt_private_zap_message(key, private_zap_event)
}

fn decrypt_private_zap_message(key: [u8; 32], private_zap_event: &Event) -> Result<Event, Error> {
    let msg: &String = extract_anon_tag_message(private_zap_event)?;
    let mut splitted = msg.split('_');

    let msg: &str = splitted.next().ok_or(Error::InvalidPrivateZapMessage)?;
    let iv: &str = splitted.next().ok_or(Error::InvalidPrivateZapMessage)?;

    // IV
    let (hrp, iv) = bech32::decode(iv)?;
    if hrp != PRIVATE_ZAP_IV_BECH32_PREFIX {
        return Err(Error::WrongBech32PrefixOrVariant);
    }

    // Msg
    let (hrp, msg) = bech32::decode(msg)?;
    if hrp != PRIVATE_ZAP_MSG_BECH32_PREFIX {
        return Err(Error::WrongBech32PrefixOrVariant);
    }

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
    use super::*;

    #[test]
    fn test_encrypt_decrypt_private_zap_message() {
        let alice_keys = Keys::generate();
        let bob_keys = Keys::generate();

        let relays = [Url::parse("wss://relay.damus.io").unwrap()];
        let msg = "Private Zap message!";
        let data = ZapRequestData::new(bob_keys.public_key(), relays).message(msg);
        let private_zap = private_zap_request(data, &alice_keys).unwrap();

        let private_zap_msg = decrypt_sent_private_zap_message(
            alice_keys.secret_key(),
            &bob_keys.public_key(),
            &private_zap,
        )
        .unwrap();

        assert_eq!(msg, &private_zap_msg.content);

        let private_zap_msg =
            decrypt_received_private_zap_message(bob_keys.secret_key(), &private_zap).unwrap();

        assert_eq!(msg, &private_zap_msg.content)
    }
}
