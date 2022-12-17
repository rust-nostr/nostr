// Copyright (c) 2021 Paul Miller
// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::str::FromStr;

use anyhow::{anyhow, Result};
use bitcoin::hashes::hex::FromHex;
use bitcoin::secp256k1::schnorr::Signature;
use bitcoin::secp256k1::{Secp256k1, XOnlyPublicKey};
use serde::{Deserialize, Deserializer};

pub mod builder;
pub mod kind;
pub mod tag;

pub use self::builder::EventBuilder;
pub use self::kind::{Kind, KindBase};
pub use self::tag::{Marker, Tag, TagData, TagKind};
use crate::Sha256Hash;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Event {
    pub id: Sha256Hash, // hash of serialized event with id 0
    pub pubkey: XOnlyPublicKey,
    pub created_at: u64, // unix timestamp seconds
    pub kind: Kind,
    pub tags: Vec<Tag>,
    pub content: String,
    #[serde(deserialize_with = "sig_string")]
    pub sig: Signature,
}

fn sig_string<'de, D>(deserializer: D) -> Result<Signature, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    let sig = Signature::from_str(&s);
    sig.map_err(serde::de::Error::custom)
}

impl Event {
    /// Verify event
    pub fn verify(&self) -> Result<(), bitcoin::secp256k1::Error> {
        let secp = Secp256k1::new();
        let id = EventBuilder::gen_id(
            &self.pubkey,
            self.created_at,
            &self.kind,
            &self.tags,
            &self.content,
        );
        let message = bitcoin::secp256k1::Message::from_slice(&id)?;
        secp.verify_schnorr(&self.sig, &message, &self.pubkey)
    }

    /// New event from json string
    pub fn from_json<S>(json: S) -> Result<Self>
    where
        S: Into<String>,
    {
        let event: Self = serde_json::from_str(&json.into())?;
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
        created_at: u64,
        kind: u8,
        tags: Vec<Tag>,
        content: &str,
        sig: &str,
    ) -> Result<Self> {
        let id = Sha256Hash::from_hex(id).map_err(|e| anyhow!(e.to_string()))?;
        let pubkey = XOnlyPublicKey::from_str(pubkey)?;
        let kind = serde_json::from_str(&kind.to_string())?;
        let sig = Signature::from_str(sig)?;

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

    use crate::Keys;

    #[test]
    fn test_tags_deser_without_recommended_relay() {
        //The TAG array has dynamic length because the third element(Recommended relay url) is optional
        let sample_event = r#"{"id":"2be17aa3031bdcb006f0fce80c146dea9c1c0268b0af2398bb673365c6444d45","pubkey":"f86c44a2de95d9149b51c6a29afeabba264c18e2fa7c49de93424a0c56947785","created_at":1640839235,"kind":4,"tags":[["p","13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"]],"content":"uRuvYr585B80L6rSJiHocw==?iv=oh6LVqdsYYol3JfFnXTbPA==","sig":"a5d9290ef9659083c490b303eb7ee41356d8778ff19f2f91776c8dc4443388a64ffcf336e61af4c25c05ac3ae952d1ced889ed655b67790891222aaa15b99fdd"}"#;
        let ev_ser = Event::from_json(sample_event).unwrap();
        assert_eq!(ev_ser.as_json().unwrap(), sample_event);
    }

    #[test]
    fn test_custom_kind() {
        let keys = Keys::generate_from_os_random();
        let e: Event = EventBuilder::new(Kind::Custom(123), "my content", &[])
            .to_event(&keys)
            .unwrap();

        let serialized = e.as_json().unwrap();
        let deserialized = Event::from_json(serialized).unwrap();

        assert_eq!(e, deserialized);
        assert_eq!(Kind::Custom(123), e.kind);
        assert_eq!(Kind::Custom(123), deserialized.kind);
    }
}
