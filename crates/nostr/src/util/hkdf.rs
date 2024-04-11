// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! HKDF Util

use alloc::vec::Vec;

use bitcoin::hashes::hmac::{Hmac, HmacEngine};
use bitcoin::hashes::sha256::Hash as Sha256Hash;
use bitcoin::hashes::{Hash, HashEngine};

/// HKDF extract
#[inline]
pub fn extract(salt: &[u8], input_key_material: &[u8]) -> Hmac<Sha256Hash> {
    let mut engine: HmacEngine<Sha256Hash> = HmacEngine::new(salt);
    engine.input(input_key_material);
    Hmac::from_engine(engine)
}

/// HKDF expand
pub fn expand(prk: &[u8], info: &[u8], output_len: usize) -> Vec<u8> {
    let mut output: Vec<u8> = Vec::with_capacity(output_len);
    let mut t: Vec<u8> = Vec::with_capacity(32);

    let mut i: u8 = 1u8;
    while output.len() < output_len {
        let mut engine: HmacEngine<Sha256Hash> = HmacEngine::new(prk);

        if !t.is_empty() {
            engine.input(&t);
        }

        engine.input(info);
        engine.input(&[i]);

        t = Hmac::from_engine(engine).to_byte_array().to_vec();
        output.extend_from_slice(&t);

        i += 1;
    }

    output.truncate(output_len);
    output
}
