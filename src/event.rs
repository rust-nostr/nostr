use std::{error::Error, str::FromStr, time::{Duration, SystemTime, UNIX_EPOCH}};


use chrono::{DateTime, NaiveDateTime};
use chrono::{serde::ts_seconds, Utc};

use bitcoin_hashes::{hex::FromHex, hex::ToHex, sha256, Hash};

use secp256k1::rand::rngs::OsRng;
use secp256k1::{schnorrsig, Secp256k1};
use serde::{Serialize, Deserialize, Deserializer};
use serde_json::{Value, json};
use serde_repr::*;
use tungstenite::Message;

use crate::error::NostrError;

#[derive(Debug, PartialEq)]
pub enum NostrMessage {
    Ping,
    Notice(String),
    Event(Event),
}

pub fn handle_incoming_message(msg: Message) -> Result<NostrMessage, Box<dyn Error>> {
    let text = msg.to_text()?;

    // Ping
    if text == "PING" {
        return Ok(NostrMessage::Ping);
    }

    let v: Value = serde_json::from_str(text)?;

    // Notice
    if v[0] == "notice" {
        let notice = v[1].to_string();
        println!("message from relay: {}", notice.clone());
        return Ok(NostrMessage::Notice(notice));
    }

    // Regular events
    let event = Event::new_from_json(v[0].to_string())?;
    let _context = v[1].clone();

    Ok(NostrMessage::Event(event))
}

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
    pub fn new(content: &str) -> Self {
        let secp = Secp256k1::new();
        let mut rng = OsRng::new().expect("OsRng");
        let key_pair = schnorrsig::KeyPair::new(&secp, &mut rng);
        let pubkey = schnorrsig::PublicKey::from_keypair(&secp, &key_pair);

        // Doing all this extra work to construct a time with zero nanoseconds
        let now =
            SystemTime::now().duration_since(UNIX_EPOCH).expect("system time before Unix epoch");
        let naive = NaiveDateTime::from_timestamp(now.as_secs() as i64, 0);
        let created_at = DateTime::from_utc(naive, Utc);

        let kind = Kind::TextNote;

        let event_json = json!([0, pubkey.to_string(), created_at, kind, [], content]).to_string();

        let id = sha256::Hash::hash(&event_json.as_bytes());
        let message = secp256k1::Message::from_slice(&id.into_inner()).expect("Failed to make message");
        let sig = secp.schnorrsig_sign(&message, &key_pair);

        let event = Event {
            id,
            pubkey,
            created_at,
            kind,
            tags: vec![],
            content: content.to_string(),
            sig
        };

        dbg!(event.clone());

        event

    }

    pub fn new_dummy(
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

        Event {
            id,
            pubkey,
            created_at,
            kind,
            tags,
            content: content.to_string(),
            sig,
        }
    }

    fn new_from_json(json: String) -> Result<Self, Box<dyn Error>> {
        Ok(serde_json::from_str(&json)?)
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
