// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Unsigned Event

use alloc::string::String;

#[cfg(feature = "std")]
use secp256k1::rand::rngs::OsRng;
use secp256k1::rand::{CryptoRng, Rng};
use secp256k1::schnorr::Signature;
use secp256k1::{Message, Secp256k1, Signing, Verification};

use super::error::Error;
use crate::{Event, EventId, JsonUtil, Keys, Kind, PublicKey, Tag, Tags, Timestamp};
#[cfg(feature = "std")]
use crate::{NostrSigner, SECP256K1};

/// Unsigned event
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct UnsignedEvent {
    /// Event ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<EventId>,
    /// Author
    pub pubkey: PublicKey,
    /// Timestamp (seconds)
    pub created_at: Timestamp,
    /// Kind
    pub kind: Kind,
    /// Tag list
    pub tags: Tags,
    /// Content
    pub content: String,
}

impl UnsignedEvent {
    /// Construct new unsigned event
    pub fn new<I, S>(
        public_key: PublicKey,
        created_at: Timestamp,
        kind: Kind,
        tags: I,
        content: S,
    ) -> Self
    where
        I: IntoIterator<Item = Tag>,
        S: Into<String>,
    {
        Self {
            id: None,
            pubkey: public_key,
            created_at,
            kind,
            tags: Tags::from_list(tags.into_iter().collect()),
            content: content.into(),
        }
    }

    /// Ensure to set [EventId]
    ///
    /// If `id` is `None`, compute and set it.
    #[inline]
    pub fn ensure_id(&mut self) {
        if self.id.is_none() {
            self.id = Some(self.compute_id());
        }
    }

    /// Get the unsigned event ID
    ///
    /// If the [`EventId`] is not set, will be computed and saved in the struct.
    pub fn id(&mut self) -> EventId {
        // Ensure that the ID is set
        self.ensure_id();

        // This can't fail, because we already ensured that the ID is set
        self.id.unwrap()
    }

    #[inline]
    fn compute_id(&self) -> EventId {
        EventId::new(
            &self.pubkey,
            &self.created_at,
            &self.kind,
            &self.tags,
            &self.content,
        )
    }

    /// Verify if the [`EventId`] it's composed correctly
    pub fn verify_id(&self) -> Result<(), Error> {
        if let Some(id) = self.id {
            let computed_id: EventId = self.compute_id();
            if id != computed_id {
                return Err(Error::InvalidId);
            }
        }

        Ok(())
    }

    /// Sign an unsigned event
    #[inline]
    #[cfg(feature = "std")]
    pub async fn sign<T>(self, signer: &T) -> Result<Event, Error>
    where
        T: NostrSigner,
    {
        Ok(signer.sign_event(self).await?)
    }

    /// Sign an unsigned event with [`Keys`] signer
    ///
    /// Internally: calculate [EventId] (if not set), sign it, compose and verify [Event].
    #[inline]
    #[cfg(feature = "std")]
    pub fn sign_with_keys(self, keys: &Keys) -> Result<Event, Error> {
        self.sign_with_ctx(SECP256K1, &mut OsRng, keys)
    }

    /// Sign an unsigned event with [`Keys`] signer
    ///
    /// Internally: calculate [EventId] (if not set), sign it, compose and verify [Event].
    pub fn sign_with_ctx<C, R>(
        self,
        secp: &Secp256k1<C>,
        rng: &mut R,
        keys: &Keys,
    ) -> Result<Event, Error>
    where
        C: Signing + Verification,
        R: Rng + CryptoRng,
    {
        let verify_id: bool = self.id.is_some();
        let id: EventId = self.id.unwrap_or_else(|| self.compute_id());
        let message: Message = Message::from_digest(id.to_bytes());
        let sig: Signature = keys.sign_schnorr_with_ctx(secp, &message, rng);
        self.internal_add_signature(secp, id, sig, verify_id, false)
    }

    /// Add signature to unsigned event
    ///
    /// Internally verify the [Event].
    #[inline]
    #[cfg(feature = "std")]
    pub fn add_signature(self, sig: Signature) -> Result<Event, Error> {
        self.add_signature_with_ctx(SECP256K1, sig)
    }

    /// Add signature to unsigned event
    ///
    /// Internally verify the [Event].
    #[inline]
    pub fn add_signature_with_ctx<C>(
        self,
        secp: &Secp256k1<C>,
        sig: Signature,
    ) -> Result<Event, Error>
    where
        C: Verification,
    {
        let verify_id: bool = self.id.is_some();
        let id: EventId = self.id.unwrap_or_else(|| self.compute_id());
        self.internal_add_signature(secp, id, sig, verify_id, true)
    }

    fn internal_add_signature<C>(
        self,
        secp: &Secp256k1<C>,
        id: EventId,
        sig: Signature,
        verify_id: bool,
        verify_sig: bool,
    ) -> Result<Event, Error>
    where
        C: Verification,
    {
        let event: Event = Event::new(
            id,
            self.pubkey,
            self.created_at,
            self.kind,
            self.tags,
            self.content,
            sig,
        );

        // Verify event ID
        if verify_id && !event.verify_id() {
            return Err(Error::InvalidId);
        }

        // Verify event signature
        if verify_sig && !event.verify_signature_with_ctx(secp) {
            return Err(Error::InvalidSignature);
        }

        Ok(event)
    }
}

impl JsonUtil for UnsignedEvent {
    type Err = Error;
}

impl From<Event> for UnsignedEvent {
    fn from(event: Event) -> Self {
        Self {
            id: Some(event.id),
            pubkey: event.pubkey,
            created_at: event.created_at,
            kind: event.kind,
            tags: event.tags,
            content: event.content,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_id() {
        let json = r#"{"content":"uRuvYr585B80L6rSJiHocw==?iv=oh6LVqdsYYol3JfFnXTbPA==","created_at":1640839235,"kind":4,"pubkey":"f86c44a2de95d9149b51c6a29afeabba264c18e2fa7c49de93424a0c56947785","tags":[["p","13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"]]}"#;
        let mut unsigned = UnsignedEvent::from_json(json).unwrap();
        let expected_id: EventId =
            EventId::from_hex("2be17aa3031bdcb006f0fce80c146dea9c1c0268b0af2398bb673365c6444d45")
                .unwrap();

        assert!(unsigned.id.is_none());
        assert_eq!(unsigned.id(), expected_id);
    }

    #[test]
    fn test_deserialize_unsigned_event_with_id() {
        let json = r#"{"content":"uRuvYr585B80L6rSJiHocw==?iv=oh6LVqdsYYol3JfFnXTbPA==","created_at":1640839235,"id":"2be17aa3031bdcb006f0fce80c146dea9c1c0268b0af2398bb673365c6444d45","kind":4,"pubkey":"f86c44a2de95d9149b51c6a29afeabba264c18e2fa7c49de93424a0c56947785","tags":[["p","13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"]]}"#;
        let event_id: EventId =
            EventId::from_hex("2be17aa3031bdcb006f0fce80c146dea9c1c0268b0af2398bb673365c6444d45")
                .unwrap();
        let unsigned = UnsignedEvent::from_json(json).unwrap();
        assert_eq!(unsigned.id, Some(event_id));
        assert_eq!(
            unsigned.content,
            "uRuvYr585B80L6rSJiHocw==?iv=oh6LVqdsYYol3JfFnXTbPA=="
        );
        assert_eq!(unsigned.kind, Kind::EncryptedDirectMessage);
    }

    #[test]
    fn test_deserialize_unsigned_event_without_id() {
        let json = r#"{"content":"uRuvYr585B80L6rSJiHocw==?iv=oh6LVqdsYYol3JfFnXTbPA==","created_at":1640839235,"kind":4,"pubkey":"f86c44a2de95d9149b51c6a29afeabba264c18e2fa7c49de93424a0c56947785","tags":[["p","13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"]]}"#;
        let unsigned = UnsignedEvent::from_json(json).unwrap();
        assert_eq!(unsigned.id, None);
        assert_eq!(
            unsigned.content,
            "uRuvYr585B80L6rSJiHocw==?iv=oh6LVqdsYYol3JfFnXTbPA=="
        );
        assert_eq!(unsigned.kind, Kind::EncryptedDirectMessage);
    }
}
