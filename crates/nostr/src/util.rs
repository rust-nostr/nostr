// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Util

use alloc::string::{String, ToString};
use core::str::FromStr;

use bitcoin::secp256k1::{ecdh, Error, PublicKey, SecretKey, XOnlyPublicKey};
#[cfg(feature = "std")]
use bitcoin::secp256k1::{rand, All, Secp256k1};
#[cfg(feature = "std")]
use once_cell::sync::Lazy;

/// Generate shared key
///
/// **Important: use of a strong cryptographic hash function may be critical to security! Do NOT use
/// unless you understand cryptographical implications.**
pub fn generate_shared_key(sk: &SecretKey, pk: &XOnlyPublicKey) -> Result<[u8; 32], Error> {
    let pk_normalized: PublicKey = normalize_schnorr_pk(pk)?;
    let ssp: [u8; 64] = ecdh::shared_secret_point(&pk_normalized, sk);
    let mut shared_key: [u8; 32] = [0u8; 32];
    shared_key.copy_from_slice(&ssp[..32]);
    Ok(shared_key)
}

/// Normalize Schnorr public key
fn normalize_schnorr_pk(schnorr_pk: &XOnlyPublicKey) -> Result<PublicKey, Error> {
    let mut pk = String::from("02");
    pk.push_str(&schnorr_pk.to_string());
    PublicKey::from_str(&pk)
}

/// Secp256k1 global context
#[cfg(feature = "std")]
pub static SECP256K1: Lazy<Secp256k1<All>> = Lazy::new(|| {
    let mut ctx = Secp256k1::new();
    let mut rng = rand::thread_rng();
    ctx.randomize(&mut rng);
    ctx
});
