use std::{
    error::Error,
    str::FromStr,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use chrono::{serde::ts_seconds, Utc};
use chrono::{DateTime, NaiveDateTime};

use bitcoin_hashes::{hex::FromHex, sha256, Hash};

use secp256k1::{schnorrsig, Secp256k1};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::json;
use serde_repr::*;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Event {
    id: sha256::Hash, // hash of serialized event with id 0
    pubkey: schnorrsig::PublicKey,
    #[serde(with = "ts_seconds")]
    created_at: DateTime<Utc>, // unix timestamp seconds
    kind: Kind,
    tags: Vec<Tag>,
    content: String,
    #[serde(deserialize_with = "sig_string")] // Serde derive is being weird
    sig: schnorrsig::Signature,
}

fn sig_string<'de, D>(deserializer: D) -> Result<schnorrsig::Signature, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    let sig = schnorrsig::Signature::from_str(&s);
    sig.map_err(serde::de::Error::custom)
}

impl Event {
    fn gen_id(
        pubkey: &schnorrsig::PublicKey,
        created_at: &DateTime<Utc>,
        kind: &Kind,
        tags: &Vec<Tag>,
        content: &str,
    ) -> sha256::Hash {
        let event_json =
            json!([0, pubkey, created_at.timestamp(), kind, tags, content]).to_string();
        sha256::Hash::hash(&event_json.as_bytes())
    }
    /// Create a new TextNote Event
    pub fn new_textnote(
        content: &str,
        keypair: &schnorrsig::KeyPair,
    ) -> Result<Self, Box<dyn Error>> {
        let secp = Secp256k1::new();
        let pubkey = schnorrsig::PublicKey::from_keypair(&secp, keypair);

        // Doing all this extra work to construct a DateTime with zero nanoseconds
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before Unix epoch");
        let naive = NaiveDateTime::from_timestamp(now.as_secs() as i64, 0);
        let created_at = DateTime::from_utc(naive, Utc);

        // TODO: support more event kinds
        let kind = Kind::TextNote;

        // For some reason the timestamp isn't serializing correctly so I do it manually
        let id = Self::gen_id(&pubkey, &created_at, &kind, &vec![], content);

        // let m1 = Message::from_hashed_data::<sha256::Hash>("Hello world!".as_bytes());
        // is equivalent to
        // let m2 = Message::from(sha256::Hash::hash("Hello world!".as_bytes()));

        let message = secp256k1::Message::from(id);

        // Let the schnorr library handle the aux for us
        // I _think_ this is bip340 compliant
        let sig = secp.schnorrsig_sign(&message, &keypair);

        let event = Event {
            id,
            pubkey,
            created_at,
            kind,
            tags: vec![],
            content: content.to_string(),
            sig,
        };

        // This isn't failing so that's a good thing, yes?
        match event.verify() {
            Ok(()) => Ok(event),
            Err(e) => Err(Box::new(e)),
        }
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
        let message = secp256k1::Message::from(id);
        secp.schnorrsig_verify(&self.sig, &message, &self.pubkey)
    }

    /// This is just for serde sanity checking
    #[allow(dead_code)]
    pub(crate) fn new_dummy(
        id: &str,
        pubkey: &str,
        created_at: u32,
        kind: u8,
        tags: Vec<Tag>,
        content: &str,
        sig: &str,
    ) -> Self {
        let id = sha256::Hash::from_hex(id).unwrap();
        let pubkey = schnorrsig::PublicKey::from_str(pubkey).unwrap();
        let created_at = DateTime::<Utc>::from(UNIX_EPOCH + Duration::new(created_at as u64, 0));
        let kind = serde_json::from_str(&kind.to_string()).unwrap();
        let sig = schnorrsig::Signature::from_str(sig).unwrap();

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
            event
        } else {
            panic!("didn't verify!")
        }
    }

    pub fn new_from_json(json: String) -> Result<Self, Box<dyn Error>> {
        Ok(serde_json::from_str(&json)?)
    }

    pub fn as_json(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug, Copy, Clone)]
#[repr(u8)]
pub enum Kind {
    SetMetadata = 0,
    TextNote = 1,
    RecommendServer = 2,
    ContactList = 3,
    EncryptedDirectMessage = 4,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Tag {}
