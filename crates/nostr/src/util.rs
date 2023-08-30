// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Util

use core::str::FromStr;

use secp256k1::{ecdh, PublicKey, SecretKey, XOnlyPublicKey};

/// Generate shared key
///
/// **Important: use of a strong cryptographic hash function may be critical to security! Do NOT use
/// unless you understand cryptographical implications.**
pub fn generate_shared_key(
    sk: &SecretKey,
    pk: &XOnlyPublicKey,
) -> Result<[u8; 32], secp256k1::Error> {
    let pk_normalized: PublicKey = normalize_schnorr_pk(pk)?;
    let ssp: [u8; 64] = ecdh::shared_secret_point(&pk_normalized, sk);
    let mut shared_key: [u8; 32] = [0u8; 32];
    shared_key.copy_from_slice(&ssp[..32]);
    Ok(shared_key)
}

/// Normalize Schnorr public key
fn normalize_schnorr_pk(schnorr_pk: &XOnlyPublicKey) -> Result<PublicKey, secp256k1::Error> {
    let mut pk = String::from("02");
    pk.push_str(&schnorr_pk.to_string());
    PublicKey::from_str(&pk)
}
