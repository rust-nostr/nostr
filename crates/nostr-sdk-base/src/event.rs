// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::fmt;
use std::str::FromStr;

use anyhow::{anyhow, Result};
use bitcoin_hashes::hex::FromHex;
use bitcoin_hashes::{sha256, Hash};
use chrono::serde::ts_seconds;
use chrono::DateTime;
use chrono::{TimeZone, Utc};
use secp256k1::{schnorr, Secp256k1, XOnlyPublicKey};
use serde::{Deserialize, Deserializer};
use serde_json::json;
use serde_repr::*;

use crate::util::nip04;
use crate::Keys;

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct Event {
    pub id: sha256::Hash, // hash of serialized event with id 0
    pub pubkey: XOnlyPublicKey,
    #[serde(with = "ts_seconds")]
    pub created_at: DateTime<Utc>, // unix timestamp seconds
    pub kind: Kind,
    pub tags: Vec<Tag>,
    pub content: String,
    #[serde(deserialize_with = "sig_string")] // Serde derive is being weird
    pub sig: schnorr::Signature,
}

fn sig_string<'de, D>(deserializer: D) -> Result<schnorr::Signature, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    let sig = schnorr::Signature::from_str(&s);
    sig.map_err(serde::de::Error::custom)
}

impl Event {
    fn gen_id(
        pubkey: &XOnlyPublicKey,
        created_at: &DateTime<Utc>,
        kind: &Kind,
        tags: &Vec<Tag>,
        content: &str,
    ) -> sha256::Hash {
        let event_json =
            json!([0, pubkey, created_at.timestamp(), kind, tags, content]).to_string();
        sha256::Hash::hash(event_json.as_bytes())
    }

    fn time_now() -> DateTime<Utc> {
        // Return current DateTime with no nanos
        Utc.timestamp(Utc::now().timestamp(), 0)
    }

    /// Create a generic type of event
    pub fn new_generic(content: &str, keys: &Keys, tags: &Vec<Tag>, kind: Kind) -> Result<Self> {
        let secp = Secp256k1::new();

        let keypair = &keys.key_pair()?;
        let pubkey = XOnlyPublicKey::from_keypair(keypair).0;

        let created_at = Self::time_now();

        let id = Self::gen_id(&pubkey, &created_at, &kind, tags, content);

        // Message::from_hashed_data::<sha256::Hash>("Hello world!".as_bytes());
        // is equivalent to
        // Message::from(sha256::Hash::hash("Hello world!".as_bytes()));

        let message = secp256k1::Message::from_slice(&id)?;

        // Let the schnorr library handle the aux for us
        // I _think_ this is bip340 compliant
        let sig = secp.sign_schnorr(&message, keypair);

        let event = Event {
            id,
            pubkey,
            created_at,
            kind,
            tags: tags.clone(),
            content: content.to_string(),
            sig,
        };

        // This isn't failing so that's a good thing, yes?
        match event.verify() {
            Ok(()) => Ok(event),
            Err(e) => Err(anyhow!(e)),
        }
    }

    /// Create a new TextNote Event
    pub fn new_textnote(content: &str, keys: &Keys, tags: &Vec<Tag>) -> Result<Self> {
        Self::new_generic(content, keys, tags, Kind::Base(KindBase::TextNote))
    }

    pub fn backup_contacts(keys: &Keys, list: Vec<Contact>) -> Result<Self> {
        let tags: Vec<Tag> = list
            .iter()
            .map(|contact| {
                Tag::new(TagData::ContactList {
                    pk: contact.pk,
                    relay_url: contact.relay_url.clone(),
                    alias: contact.alias.clone(),
                })
            })
            .collect();

        Self::new_generic("", keys, &tags, Kind::Base(KindBase::ContactList))
    }

    /// Create encrypted direct msg event
    pub fn new_encrypted_direct_msg(
        sender_keys: &Keys,
        receiver_keys: &Keys,
        content: &str,
    ) -> Result<Self> {
        Self::new_generic(
            &nip04::encrypt(
                &sender_keys.secret_key()?,
                &receiver_keys.public_key,
                content,
            ),
            sender_keys,
            &vec![Tag::new(TagData::EncryptedDirectMessage {
                pk: receiver_keys.public_key,
            })],
            Kind::Base(KindBase::EncryptedDirectMessage),
        )
    }

    /// Create delete event
    pub fn delete(keys: &Keys, ids: Vec<sha256::Hash>, content: Option<&str>) -> Result<Self> {
        let tags: Vec<Tag> = ids
            .iter()
            .map(|id| Tag::new(TagData::EventId(id.to_string())))
            .collect();

        Self::new_generic(
            content.unwrap_or(""),
            keys,
            &tags,
            Kind::Base(KindBase::EventDeletion),
        )
    }

    pub fn verify(&self) -> Result<(), secp256k1::Error> {
        let secp = Secp256k1::new();
        let id = Self::gen_id(
            &self.pubkey,
            &self.created_at,
            &self.kind,
            &self.tags,
            &self.content,
        );
        let message = secp256k1::Message::from_slice(&id)?;
        secp.verify_schnorr(&self.sig, &message, &self.pubkey)
    }

    /// This is just for serde sanity checking
    #[allow(dead_code)]
    pub(crate) fn new_dummy(
        id: &str,
        pubkey: &str,
        created_at: i64,
        kind: u8,
        tags: Vec<Tag>,
        content: &str,
        sig: &str,
    ) -> Result<Self> {
        let id = sha256::Hash::from_hex(id)?;
        let pubkey = XOnlyPublicKey::from_str(pubkey)?;
        let created_at = Utc.timestamp(created_at, 0);
        let kind = serde_json::from_str(&kind.to_string())?;
        let sig = schnorr::Signature::from_str(sig)?;

        let event = Event {
            id,
            pubkey,
            created_at,
            kind,
            tags,
            content: content.to_string(),
            sig,
        };

        if event.verify().is_ok() {
            Ok(event)
        } else {
            Err(anyhow!("Didn't verify"))
        }
    }

    pub fn new_from_json(json: String) -> Result<Self> {
        Ok(serde_json::from_str(&json)?)
    }

    pub fn as_json(&self) -> String {
        // This shouldn't be able to fail
        serde_json::to_string(&self).expect("Failed to serialize to json")
    }
}

#[derive(Serialize_repr, Deserialize_repr, Eq, PartialEq, Debug, Copy, Clone)]
#[repr(u8)]
pub enum KindBase {
    Metadata = 0,
    TextNote = 1,
    RecommendRelay = 2,
    ContactList = 3,
    EncryptedDirectMessage = 4,
    EventDeletion = 5,
    Boost = 6,
    Reaction = 7,
    ChannelCreation = 40,
    ChannelMetadata = 41,
    ChannelMessage = 42,
    ChannelHideMessage = 43,
    ChannelMuteUser = 44,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Copy, Clone)]
#[serde(untagged)]
pub enum Kind {
    Base(KindBase),
    Custom(u16),
}

pub enum TagKind {
    P,
    E,
    Nonce,
}

impl fmt::Display for TagKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::P => write!(f, "p"),
            Self::E => write!(f, "e"),
            Self::Nonce => write!(f, "nonce"),
        }
    }
}

pub enum TagData {
    Generic(TagKind, Vec<String>),
    EventId(String),
    ContactList {
        pk: XOnlyPublicKey,
        relay_url: String,
        alias: String,
    },
    EncryptedDirectMessage {
        pk: XOnlyPublicKey,
    },
    POW {
        nonce: u128,
        difficulty: u8,
    },
}

impl From<TagData> for Vec<String> {
    fn from(data: TagData) -> Self {
        match data {
            TagData::Generic(kind, data) => vec![vec![kind.to_string()], data].concat(),
            TagData::EventId(id) => vec![TagKind::E.to_string(), id],
            TagData::ContactList {
                pk,
                relay_url,
                alias,
            } => vec![TagKind::P.to_string(), pk.to_string(), relay_url, alias],
            TagData::EncryptedDirectMessage { pk } => vec![TagKind::P.to_string(), pk.to_string()],
            TagData::POW { nonce, difficulty } => vec![
                TagKind::Nonce.to_string(),
                nonce.to_string(),
                difficulty.to_string(),
            ],
        }
    }
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct Tag(Vec<String>);

impl Tag {
    pub fn new(data: TagData) -> Self {
        Self(data.into())
    }

    pub fn kind(&self) -> &str {
        &self.0[0]
    }

    pub fn content(&self) -> &str {
        &self.0[1]
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub struct Contact {
    pub alias: String,
    pub pk: XOnlyPublicKey,
    pub relay_url: String,
}

impl Contact {
    pub fn new(alias: &str, pk: XOnlyPublicKey, relay_url: &str) -> Self {
        Self {
            alias: alias.into(),
            pk,
            relay_url: relay_url.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tags_deser_without_recommended_relay() {
        //The TAG array has dynamic length because the third element(Recommended relay url) is optional
        let sample_event = r#"{"id":"2be17aa3031bdcb006f0fce80c146dea9c1c0268b0af2398bb673365c6444d45","pubkey":"f86c44a2de95d9149b51c6a29afeabba264c18e2fa7c49de93424a0c56947785","created_at":1640839235,"kind":4,"tags":[["p","13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"]],"content":"uRuvYr585B80L6rSJiHocw==?iv=oh6LVqdsYYol3JfFnXTbPA==","sig":"a5d9290ef9659083c490b303eb7ee41356d8778ff19f2f91776c8dc4443388a64ffcf336e61af4c25c05ac3ae952d1ced889ed655b67790891222aaa15b99fdd"}"#;
        let ev_ser = Event::new_from_json(sample_event.into()).unwrap();
        assert_eq!(ev_ser.as_json(), sample_event);
    }

    #[test]
    fn test_custom_kind() {
        let keys = Keys::generate_from_os_random();
        let e = Event::new_generic("my content", &keys, &vec![], Kind::Custom(123)).unwrap();

        let serialized = e.as_json();
        let deserialized = Event::new_from_json(serialized).unwrap();

        assert_eq!(e, deserialized);
        assert_eq!(Kind::Custom(123), e.kind);
        assert_eq!(Kind::Custom(123), deserialized.kind);
    }
}
