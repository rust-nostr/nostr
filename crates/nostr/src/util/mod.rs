// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Util

use alloc::boxed::Box;
#[cfg(feature = "rand")]
use alloc::string::String;
use core::convert::Infallible;
use core::future::Future;
use core::pin::Pin;
#[cfg(feature = "std")]
use std::sync::LazyLock;

#[cfg(feature = "rand")]
use rand::Rng;
#[cfg(all(feature = "std", feature = "os-rng"))]
use rand::rand_core::UnwrapErr;
#[cfg(feature = "os-rng")]
use rand::rngs::SysRng;
#[cfg(feature = "std")]
use secp256k1::{All, Secp256k1};
use secp256k1::{Parity, PublicKey as NormalizedPublicKey, XOnlyPublicKey, ecdh};

#[cfg(feature = "nip44")]
pub mod hkdf;
mod json;

pub(crate) use self::json::{impl_json_methods, parse_json, parse_json_from_value};
use crate::error::Error;
use crate::key::{PublicKey, SecretKey};

/// A boxed future
#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
pub type BoxedFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// A boxed future
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
pub type BoxedFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;

#[cfg(feature = "rand")]
fn random_bytes<R, const N: usize>(rng: &mut R) -> [u8; N]
where
    R: Rng,
{
    let mut ret: [u8; N] = [0u8; N];
    rng.fill_bytes(&mut ret);
    ret
}

#[inline]
#[cfg(feature = "rand")]
pub(crate) fn random_32_bytes<R>(rng: &mut R) -> [u8; 32]
where
    R: Rng,
{
    random_bytes(rng)
}

#[cfg(feature = "rand")]
pub(crate) fn random_hex_string<R, const N: usize>(rng: &mut R) -> String
where
    R: Rng,
{
    let bytes: [u8; N] = random_bytes(rng);
    faster_hex::hex_string(&bytes)
}

/// Generate shared key
///
/// **Important: use of a strong cryptographic hash function may be critical to security! Do NOT use
/// unless you understand cryptographical implications.**
pub fn generate_shared_key(
    secret_key: &SecretKey,
    public_key: &PublicKey,
) -> Result<[u8; 32], Error> {
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
pub static SECP256K1: LazyLock<Secp256k1<All>> = LazyLock::new(|| {
    #[cfg(feature = "os-rng")]
    let mut ctx: Secp256k1<All> = Secp256k1::new();
    #[cfg(not(feature = "os-rng"))]
    let ctx: Secp256k1<All> = Secp256k1::new();

    // Randomize
    #[cfg(feature = "os-rng")]
    {
        let seed: [u8; 32] = random_32_bytes(&mut UnwrapErr(SysRng));
        ctx.seeded_randomize(&seed);
    }

    ctx
});

pub(crate) trait UnwrapInfallible<T>: Sized {
    fn unwrap_infallible(self) -> T;
}

impl<T> UnwrapInfallible<T> for Result<T, Infallible> {
    #[inline]
    fn unwrap_infallible(self) -> T {
        match self {
            Ok(value) => value,
            Err(e) => match e {},
        }
    }
}
