// Copyright (c) 2021 Paul Miller
// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::str::FromStr;
use std::time::Instant;

use anyhow::{anyhow, Result};
use bitcoin_hashes::hex::FromHex;
use bitcoin_hashes::{sha256, Hash};
use chrono::serde::ts_seconds;
use chrono::DateTime;
use chrono::{TimeZone, Utc};
use secp256k1::{schnorr, KeyPair, Secp256k1, XOnlyPublicKey};
use serde::{Deserialize, Deserializer};
use serde_json::{json, Value};
use url::Url;

pub mod kind;
pub mod tag;

pub use self::kind::{Kind, KindBase};
pub use self::tag::{Tag, TagData, TagKind};
use crate::util::{nip04, nip13};
use crate::{Contact, Keys};

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct Event {
    pub id: sha256::Hash, // hash of serialized event with id 0
    pub pubkey: XOnlyPublicKey,
    #[serde(with = "ts_seconds")]
    pub created_at: DateTime<Utc>, // unix timestamp seconds
    pub kind: Kind,
    pub tags: Vec<Tag>,
    pub content: String,
    #[serde(deserialize_with = "sig_string")]
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
        tags: &[Tag],
        content: &str,
    ) -> sha256::Hash {
        let json: Value = json!([0, pubkey, created_at.timestamp(), kind, tags, content]);
        let event_str: String = json.to_string();
        sha256::Hash::hash(event_str.as_bytes())
    }

    /// Create a generic type of event
    pub fn new_generic(keys: &Keys, kind: Kind, content: &str, tags: &[Tag]) -> Result<Self> {
        let secp = Secp256k1::new();
        let keypair: &KeyPair = &keys.key_pair()?;
        let pubkey: XOnlyPublicKey = keys.public_key();
        let created_at: DateTime<Utc> = Utc.timestamp(Utc::now().timestamp(), 0);

        let id: sha256::Hash = Self::gen_id(&pubkey, &created_at, &kind, tags, content);
        let message = secp256k1::Message::from_slice(&id)?;

        Ok(Self {
            id,
            pubkey,
            created_at,
            kind,
            tags: tags.to_vec(),
            content: content.to_string(),
            sig: secp.sign_schnorr(&message, keypair),
        })
    }

    /// Set metadata
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    ///
    /// # Example
    /// ```rust
    /// use nostr_sdk_base::key::{FromBech32, Keys};
    /// use nostr_sdk_base::Event;
    ///
    /// let my_keys = Keys::from_bech32("nsec1...").unwrap();
    ///
    /// let event = Event::set_metadata(
    ///     &my_keys,
    ///     Some("username"),
    ///     Some("Username"),
    ///     Some("Description"),
    ///     Some("https://example.com/avatar.png"),
    /// ).unwrap();
    /// ```
    pub fn set_metadata(
        keys: &Keys,
        username: Option<&str>,
        display_name: Option<&str>,
        about: Option<&str>,
        picture: Option<&str>,
    ) -> Result<Self> {
        let metadata: Value = json!({
            "name": username.unwrap_or(""),
            "display_name": display_name.unwrap_or(""),
            "about": about.unwrap_or(""),
            "picture": picture.unwrap_or(""),
        });

        Self::new_generic(
            keys,
            Kind::Base(KindBase::Metadata),
            &metadata.to_string(),
            &Vec::new(),
        )
    }

    ///  Add recommended relay
    pub fn add_recommended_relay(keys: &Keys, url: &Url) -> Result<Self> {
        Self::new_generic(
            keys,
            Kind::Base(KindBase::RecommendRelay),
            url.as_ref(),
            &[],
        )
    }

    /// Text note
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    ///
    /// # Example
    /// ```rust
    /// use nostr_sdk_base::key::{FromBech32, Keys};
    /// use nostr_sdk_base::Event;
    ///
    /// let my_keys = Keys::from_bech32("nsec1...").unwrap();
    ///
    /// let event = Event::new_text_note(&my_keys, "My first text note from Nostr SDK!", &[]).unwrap();
    /// ```
    pub fn new_text_note(keys: &Keys, content: &str, tags: &[Tag]) -> Result<Self> {
        Self::new_generic(keys, Kind::Base(KindBase::TextNote), content, tags)
    }

    /// POW Text note
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/13.md>
    ///
    /// # Example
    /// ```rust
    /// use nostr_sdk_base::key::{FromBech32, Keys};
    /// use nostr_sdk_base::Event;
    ///
    /// let my_keys = Keys::from_bech32("nsec1...").unwrap();
    ///
    /// let event = Event::new_pow_text_note(&my_keys, "My first POW text note from Nostr SDK!", &[], 20).unwrap();
    /// ```
    pub fn new_pow_text_note(
        keys: &Keys,
        content: &str,
        tags: &[Tag],
        difficulty: u8,
    ) -> Result<Self> {
        let mut nonce: u128 = 0;
        #[allow(unused_assignments)]
        let mut tags: Vec<Tag> = tags.to_vec();

        let pubkey = keys.public_key();
        let kind = Kind::Base(KindBase::TextNote);

        let now = Instant::now();

        loop {
            nonce += 1;

            tags.push(Tag::new(TagData::POW { nonce, difficulty }));

            let created_at: DateTime<Utc> = Utc.timestamp(Utc::now().timestamp(), 0);
            let id: sha256::Hash = Self::gen_id(&pubkey, &created_at, &kind, &tags, content);

            if nip13::get_leading_zero_bits(id) >= difficulty {
                log::debug!(
                    "{} iterations in {} ms. Avg rate {} hashes/second",
                    nonce,
                    now.elapsed().as_millis(),
                    nonce * 1000 / std::cmp::max(1, now.elapsed().as_millis())
                );

                let secp = Secp256k1::new();
                let keypair: &KeyPair = &keys.key_pair()?;
                let message = secp256k1::Message::from_slice(&id)?;

                return Ok(Self {
                    id,
                    pubkey,
                    created_at,
                    kind,
                    tags,
                    content: content.to_string(),
                    sig: secp.sign_schnorr(&message, keypair),
                });
            }

            tags.pop();
        }
    }

    /// Set contact list
    pub fn set_contact_list(keys: &Keys, list: Vec<Contact>) -> Result<Self> {
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

        Self::new_generic(keys, Kind::Base(KindBase::ContactList), "", &tags)
    }

    /// Create encrypted direct msg event
    pub fn new_encrypted_direct_msg(
        sender_keys: &Keys,
        receiver_keys: &Keys,
        content: &str,
    ) -> Result<Self> {
        Self::new_generic(
            sender_keys,
            Kind::Base(KindBase::EncryptedDirectMessage),
            &nip04::encrypt(
                &sender_keys.secret_key()?,
                &receiver_keys.public_key(),
                content,
            )?,
            &[Tag::new(TagData::PubKey(receiver_keys.public_key()))],
        )
    }

    /// Create delete event
    pub fn delete(keys: &Keys, ids: Vec<sha256::Hash>, content: Option<&str>) -> Result<Self> {
        let tags: Vec<Tag> = ids
            .iter()
            .map(|id| Tag::new(TagData::EventId(*id)))
            .collect();

        Self::new_generic(
            keys,
            Kind::Base(KindBase::EventDeletion),
            content.unwrap_or(""),
            &tags,
        )
    }

    /// Add reaction (like/upvote, dislike/downvote) to an event
    pub fn new_reaction(keys: &Keys, event: &Event, positive: bool) -> Result<Self> {
        let tags: &[Tag] = &[
            Tag::new(TagData::EventId(event.id)),
            Tag::new(TagData::PubKey(event.pubkey)),
        ];

        let content: &str = match positive {
            true => "+",
            false => "-",
        };

        Self::new_generic(keys, Kind::Base(KindBase::Reaction), content, tags)
    }

    /// Verify event
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

    /// New event from json string
    pub fn from_json(json: String) -> Result<Self> {
        let event: Self = serde_json::from_str(&json)?;
        event.verify()?;
        Ok(event)
    }

    /// Get event as json string
    pub fn as_json(&self) -> Result<String> {
        Ok(serde_json::to_string(&self)?)
    }
}

impl Event {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tags_deser_without_recommended_relay() {
        //The TAG array has dynamic length because the third element(Recommended relay url) is optional
        let sample_event = r#"{"id":"2be17aa3031bdcb006f0fce80c146dea9c1c0268b0af2398bb673365c6444d45","pubkey":"f86c44a2de95d9149b51c6a29afeabba264c18e2fa7c49de93424a0c56947785","created_at":1640839235,"kind":4,"tags":[["p","13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"]],"content":"uRuvYr585B80L6rSJiHocw==?iv=oh6LVqdsYYol3JfFnXTbPA==","sig":"a5d9290ef9659083c490b303eb7ee41356d8778ff19f2f91776c8dc4443388a64ffcf336e61af4c25c05ac3ae952d1ced889ed655b67790891222aaa15b99fdd"}"#;
        let ev_ser = Event::from_json(sample_event.into()).unwrap();
        assert_eq!(ev_ser.as_json().unwrap(), sample_event);
    }

    #[test]
    fn test_custom_kind() {
        let keys = Keys::generate_from_os_random();
        let e = Event::new_generic(&keys, Kind::Custom(123), "my content", &vec![]).unwrap();

        let serialized = e.as_json().unwrap();
        let deserialized = Event::from_json(serialized).unwrap();

        assert_eq!(e, deserialized);
        assert_eq!(Kind::Custom(123), e.kind);
        assert_eq!(Kind::Custom(123), deserialized.kind);
    }
}
