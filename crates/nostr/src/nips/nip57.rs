// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP57
//!
//! <https://github.com/nostr-protocol/nips/blob/master/57.md>

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt;

#[cfg(feature = "std")]
use aes::cipher::block_padding::Pkcs7;
#[cfg(feature = "std")]
use aes::cipher::{BlockEncryptMut, KeyIvInit};
#[cfg(feature = "std")]
use bitcoin::bech32::{self, ToBase32, Variant};
use bitcoin::hashes::sha256::Hash as Sha256Hash;
use bitcoin::hashes::Hash;
use bitcoin::secp256k1::{self, SecretKey, XOnlyPublicKey};

use super::nip01::Coordinate;
#[cfg(feature = "std")]
use super::nip04::{self, Aes256CbcEnc};
use crate::event::builder::Error as BuilderError;
use crate::key::Error as KeyError;
#[cfg(feature = "std")]
use crate::{util, Event, EventBuilder, Keys, Kind};
use crate::{EventId, Tag, Timestamp, UncheckedUrl};

#[allow(missing_docs)]
#[derive(Debug)]
pub enum Error {
    Key(KeyError),
    Builder(BuilderError),
    #[cfg(feature = "std")]
    NIP04(nip04::Error),
    #[cfg(feature = "std")]
    Bech32(bech32::Error),
    Secp256k1(secp256k1::Error),
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Key(e) => write!(f, "{e}"),
            Self::Builder(e) => write!(f, "{e}"),
            #[cfg(feature = "std")]
            Self::NIP04(e) => write!(f, "{e}"),
            #[cfg(feature = "std")]
            Self::Bech32(e) => write!(f, "{e}"),
            Self::Secp256k1(e) => write!(f, "{e}"),
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

#[cfg(feature = "std")]
impl From<nip04::Error> for Error {
    fn from(e: nip04::Error) -> Self {
        Self::NIP04(e)
    }
}

#[cfg(feature = "std")]
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
    pub fn new(public_key: XOnlyPublicKey, relays: Vec<UncheckedUrl>) -> Self {
        Self {
            public_key,
            relays,
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
    let created_at: Timestamp = Timestamp::now();
    let secret_key: SecretKey =
        create_encryption_key(keys.secret_key()?, data.public_key, created_at)?;
    let keys: Keys = Keys::new(secret_key);

    let msg: String = encrypt_private_zap_message(secret_key, data.public_key, &data.message)?;
    let mut tags: Vec<Tag> = data.into();
    tags.push(Tag::Anon { msg: Some(msg) });
    Ok(EventBuilder::new(Kind::ZapRequest, "", tags).to_event(&keys)?)
}

/// Create NIP57 encryption key for **private** zap
pub fn create_encryption_key(
    secret_key: SecretKey,
    public_key: XOnlyPublicKey,
    created_at: Timestamp,
) -> Result<SecretKey, Error> {
    let mut unhashed: String = secret_key.display_secret().to_string();
    unhashed.push_str(&public_key.to_string());
    unhashed.push_str(&created_at.to_string());
    let hash = Sha256Hash::hash(unhashed.as_bytes());
    Ok(SecretKey::from_slice(hash.as_byte_array())?)
}

#[cfg(feature = "std")]
fn encrypt_private_zap_message<T>(
    secret_key: SecretKey,
    public_key: XOnlyPublicKey,
    msg: T,
) -> Result<String, Error>
where
    T: AsRef<[u8]>,
{
    let key: [u8; 32] = util::generate_shared_key(&secret_key, &public_key);
    let iv: [u8; 16] = secp256k1::rand::random();

    let cipher = Aes256CbcEnc::new(&key.into(), &iv.into());
    let result: Vec<u8> = cipher.encrypt_padded_vec_mut::<Pkcs7>(msg.as_ref());

    // Bech32 msg
    let data = result.to_base32();
    let encrypted_bech32_msg = bech32::encode("pzap", data, Variant::Bech32)?;

    // Bech32 IV
    let data = iv.to_base32();
    let iv_bech32 = bech32::encode("iv", data, Variant::Bech32)?;

    Ok(format!("{encrypted_bech32_msg}_{iv_bech32}"))
}
