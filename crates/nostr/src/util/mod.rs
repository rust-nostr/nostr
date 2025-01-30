// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Util

use alloc::boxed::Box;
use alloc::string::String;
use core::fmt::Debug;
use core::future::Future;
use core::pin::Pin;

#[cfg(feature = "std")]
use secp256k1::global::GlobalContext;
use secp256k1::{ecdh, Parity, PublicKey as NormalizedPublicKey, XOnlyPublicKey};
use serde::de::DeserializeOwned;
use serde::Serialize;

pub mod hex;
#[cfg(feature = "nip44")]
pub mod hkdf;

use crate::nips::nip01::Coordinate;
use crate::{key, EventId, PublicKey, SecretKey, Tag};

/// A boxed future
#[cfg(not(target_arch = "wasm32"))]
pub type BoxedFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// A boxed future
#[cfg(target_arch = "wasm32")]
pub type BoxedFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;

/// Generate shared key
///
/// **Important: use of a strong cryptographic hash function may be critical to security! Do NOT use
/// unless you understand cryptographical implications.**
pub fn generate_shared_key(
    secret_key: &SecretKey,
    public_key: &PublicKey,
) -> Result<[u8; 32], key::Error> {
    let pk: XOnlyPublicKey = public_key.xonly()?;
    let public_key_normalized: NormalizedPublicKey =
        NormalizedPublicKey::from_x_only_public_key(pk, Parity::Even);
    let ssp: [u8; 64] = ecdh::shared_secret_point(&public_key_normalized, secret_key);
    let mut shared_key: [u8; 32] = [0u8; 32];
    shared_key.copy_from_slice(&ssp[..32]);
    Ok(shared_key)
}

/// Secp256k1 global context
#[cfg(feature = "std")]
pub static SECP256K1: &GlobalContext = secp256k1::global::SECP256K1;

/// JSON util
pub trait JsonUtil: Sized + Serialize + DeserializeOwned
where
    <Self as JsonUtil>::Err: From<serde_json::Error>,
{
    /// Error
    type Err: Debug;

    /// Deserialize JSON
    #[inline]
    fn from_json<T>(json: T) -> Result<Self, Self::Err>
    where
        T: AsRef<[u8]>,
    {
        Ok(serde_json::from_slice(json.as_ref())?)
    }

    /// Serialize as JSON string
    ///
    /// This method could panic! Use `try_as_json` for error propagation.
    #[inline]
    fn as_json(&self) -> String {
        self.try_as_json().unwrap()
    }

    /// Serialize as JSON string
    #[inline]
    fn try_as_json(&self) -> Result<String, Self::Err> {
        Ok(serde_json::to_string(self)?)
    }

    /// Serialize as pretty JSON string
    ///
    /// This method could panic! Use `try_as_pretty_json` for error propagation.
    #[inline]
    fn as_pretty_json(&self) -> String {
        self.try_as_pretty_json().unwrap()
    }

    /// Serialize as pretty JSON string
    #[inline]
    fn try_as_pretty_json(&self) -> Result<String, Self::Err> {
        Ok(serde_json::to_string_pretty(self)?)
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
