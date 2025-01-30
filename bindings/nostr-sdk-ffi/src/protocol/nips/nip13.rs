// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use nostr::nips::nip13;

/// Gets the number of leading zero bits. Result is between 0 and 255.
#[uniffi::export]
pub fn get_leading_zero_bits(bytes: Vec<u8>) -> u8 {
    nip13::get_leading_zero_bits(bytes)
}

/// Returns all possible ID prefixes (hex) that have the specified number of leading zero bits.
///
/// Possible values: 0-255
#[uniffi::export]
pub fn get_prefixes_for_difficulty(leading_zero_bits: u8) -> Vec<String> {
    nip13::get_prefixes_for_difficulty(leading_zero_bits)
}
