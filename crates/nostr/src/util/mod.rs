// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Util

use alloc::borrow::Cow;
use alloc::string::String;

use bitcoin::secp256k1::{self, ecdh, Parity, PublicKey as NormalizedPublicKey, XOnlyPublicKey};
#[cfg(feature = "std")]
use bitcoin::secp256k1::{rand, All, Secp256k1};
#[cfg(feature = "std")]
use once_cell::sync::Lazy;
use serde::de::DeserializeOwned;
use serde::Serialize;

pub mod hex;
#[cfg(feature = "nip44")]
pub mod hkdf;

use crate::nips::nip01::Coordinate;
use crate::nips::nip19::Nip19Profile;
use crate::{EventId, PublicKey, SecretKey, Tag};

/// Generate shared key
///
/// **Important: use of a strong cryptographic hash function may be critical to security! Do NOT use
/// unless you understand cryptographical implications.**
pub fn generate_shared_key(
    secret_key: &SecretKey,
    public_key: &PublicKey,
) -> Result<[u8; 32], secp256k1::Error> {
    let public_key: &XOnlyPublicKey = public_key.get_xonly_public_key()?;
    let public_key_normalized: NormalizedPublicKey =
        NormalizedPublicKey::from_x_only_public_key(*public_key, Parity::Even);
    let ssp: [u8; 64] = ecdh::shared_secret_point(&public_key_normalized, secret_key);
    let mut shared_key: [u8; 32] = [0u8; 32];
    shared_key.copy_from_slice(&ssp[..32]);
    Ok(shared_key)
}

/// Secp256k1 global context
#[cfg(feature = "std")]
pub static SECP256K1: Lazy<Secp256k1<All>> = Lazy::new(|| {
    let mut ctx = Secp256k1::new();
    let mut rng = rand::thread_rng();
    ctx.randomize(&mut rng);
    ctx
});

/// JSON util
pub trait JsonUtil: Sized + Serialize + DeserializeOwned
where
    <Self as JsonUtil>::Err: From<serde_json::Error>,
{
    /// Error
    type Err;

    /// Deserialize JSON
    #[inline]
    fn from_json<T>(json: T) -> Result<Self, Self::Err>
    where
        T: AsRef<[u8]>,
    {
        Ok(serde_json::from_slice(json.as_ref())?)
    }

    /// Serialize to JSON string
    #[inline]
    fn as_json(&self) -> String {
        // TODO: remove unwrap
        serde_json::to_string(self).unwrap()
    }
}

/// Event ID or Coordinate
pub enum EventIdOrCoordinate {
    /// Event ID
    Id(EventId),
    /// Event Coordinate (`a` tag)
    Coordinate(Coordinate),
}

impl From<EventIdOrCoordinate> for Tag {
    fn from(value: EventIdOrCoordinate) -> Self {
        match value {
            EventIdOrCoordinate::Id(id) => id.into(),
            EventIdOrCoordinate::Coordinate(a) => a.into(),
        }
    }
}

impl From<EventId> for EventIdOrCoordinate {
    fn from(id: EventId) -> Self {
        Self::Id(id)
    }
}

impl From<Coordinate> for EventIdOrCoordinate {
    fn from(coordinate: Coordinate) -> Self {
        Self::Coordinate(coordinate)
    }
}

#[allow(missing_docs)]
pub trait IntoPublicKey {
    fn into_public_key(self) -> PublicKey;
}

impl IntoPublicKey for PublicKey {
    fn into_public_key(self) -> PublicKey {
        self
    }
}

impl IntoPublicKey for &PublicKey {
    fn into_public_key(self) -> PublicKey {
        self.clone()
    }
}

impl IntoPublicKey for Cow<'_, PublicKey> {
    fn into_public_key(self) -> PublicKey {
        self.into_owned()
    }
}

impl IntoPublicKey for Nip19Profile {
    fn into_public_key(self) -> PublicKey {
        self.public_key
    }
}
