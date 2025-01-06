// Copyright (c) 2024 Michael Dilger
// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use core::{cmp, iter};

use nostr::prelude::Coordinate;
use nostr::{EventId, PublicKey, SingleLetterTag, Timestamp};

const CREATED_AT_BE: usize = 8;
const KIND_BE: usize = 2;
const TAG_VALUE_PAD_LEN: usize = 182;

/// Reverse created_at and convert `u64` to big-endian byte order
#[inline]
fn reverse_and_conv_to_be64(created_at: &Timestamp) -> [u8; 8] {
    // Reverse
    let created_at: u64 = u64::MAX - created_at.as_u64();

    // Convert to big-endian
    created_at.to_be_bytes()
}

/// Extend the key with the `tag_value` (fixed len of 182 bytes)
fn extend_key_with_tag_value(key: &mut Vec<u8>, len: usize, tag_value: &str) {
    let tag_value: &[u8] = tag_value.as_bytes();
    if len <= TAG_VALUE_PAD_LEN {
        key.extend(tag_value);
        key.extend(iter::repeat(0).take(TAG_VALUE_PAD_LEN - len));
    } else {
        key.extend(&tag_value[..TAG_VALUE_PAD_LEN]);
    }
}

/// Make CreatedAt + ID index key
///
/// ## Structure
///
/// `reverse_created_at(8)` + `event_id(32)`
pub fn make_ci_index_key(created_at: &Timestamp, event_id: &[u8; EventId::LEN]) -> Vec<u8> {
    let mut key: Vec<u8> = Vec::with_capacity(CREATED_AT_BE + EventId::LEN);
    key.extend(reverse_and_conv_to_be64(created_at));
    key.extend(event_id);
    key
}

/// Make Tag + CreatedAt + ID index key (for looking up event by `tag`)
///
/// ## Structure
///
/// `tag_name(1)` + `tag_value(182)` + `reverse_created_at(8)` + `event_id(32)`
pub fn make_tc_index_key(
    tag_name: &SingleLetterTag,
    tag_value: &str,
    created_at: &Timestamp,
    event_id: &[u8; EventId::LEN],
) -> Vec<u8> {
    let mut key: Vec<u8> = Vec::with_capacity(1 + TAG_VALUE_PAD_LEN + CREATED_AT_BE + EventId::LEN);

    // Push tag name
    key.push(tag_name.as_char() as u8);

    // Add tag value
    extend_key_with_tag_value(&mut key, tag_value.len(), tag_value);

    key.extend(reverse_and_conv_to_be64(created_at));
    key.extend(event_id);
    key
}

/// Make Author + CreatedAt + ID index key (for looking up event by `author`)
///
/// ## Structure
///
/// `author(32)` + `reverse_created_at(8)` + `event_id(32)`
pub fn make_ac_index_key(
    author: &[u8; PublicKey::LEN],
    created_at: &Timestamp,
    event_id: &[u8; EventId::LEN],
) -> Vec<u8> {
    let mut key: Vec<u8> = Vec::with_capacity(PublicKey::LEN + CREATED_AT_BE + EventId::LEN);
    key.extend(author);
    key.extend(reverse_and_conv_to_be64(created_at));
    key.extend(event_id);
    key
}

/// Make Author + Kind + CreatedAt + ID index key (for looking up event by `author` and `kind`)
///
/// ## Structure
///
/// `author(32)` + `kind(2)` + `reverse_created_at(8)` + `event_id(32)`
pub fn make_akc_index_key(
    author: &[u8; PublicKey::LEN],
    kind: u16,
    created_at: &Timestamp,
    event_id: &[u8; EventId::LEN],
) -> Vec<u8> {
    let mut key: Vec<u8> =
        Vec::with_capacity(PublicKey::LEN + KIND_BE + CREATED_AT_BE + EventId::LEN);
    key.extend(author);
    key.extend(kind.to_be_bytes());
    key.extend(reverse_and_conv_to_be64(created_at));
    key.extend(event_id);
    key
}

/// Make Author + Tag + CreatedAt + ID index key (for looking up event by `author` and `tag`)
///
/// ## Structure
///
/// `author(32)` + `tag_name(1)` + `tag_value(182)` + `reverse_created_at(8)` + `event_id(32)`
pub fn make_atc_index_key(
    author: &[u8; PublicKey::LEN],
    tag_name: &SingleLetterTag,
    tag_value: &str,
    created_at: &Timestamp,
    event_id: &[u8; EventId::LEN],
) -> Vec<u8> {
    let mut key: Vec<u8> =
        Vec::with_capacity(PublicKey::LEN + 1 + TAG_VALUE_PAD_LEN + CREATED_AT_BE + EventId::LEN);

    // Add author
    key.extend(author);

    // Add tag name
    key.push(tag_name.as_char() as u8);

    // Add tag value
    extend_key_with_tag_value(&mut key, tag_value.len(), tag_value);

    // Add reverse created at
    key.extend(reverse_and_conv_to_be64(created_at));

    // Add event ID
    key.extend(event_id);

    key
}

/// Make Kind + Tag + CreatedAt + ID index (for looking up event by `kind` and `tag`)
///
/// ## Structure
///
/// `kind(2)` + `tag_name(1)` + `tag_value(182)` + `reverse_created_at(8)` + `event_id(32)`
pub fn make_ktc_index_key(
    kind: u16,
    tag_name: &SingleLetterTag,
    tag_value: &str,
    created_at: &Timestamp,
    event_id: &[u8; EventId::LEN],
) -> Vec<u8> {
    let mut key: Vec<u8> =
        Vec::with_capacity(KIND_BE + TAG_VALUE_PAD_LEN + CREATED_AT_BE + EventId::LEN);
    key.extend(kind.to_be_bytes());
    key.push(tag_name.as_char() as u8);
    extend_key_with_tag_value(&mut key, tag_value.len(), tag_value);
    key.extend(reverse_and_conv_to_be64(created_at));
    key.extend(event_id);
    key
}

/// Make coordinate index key
///
/// ## Structure
///
/// `kind(2)` + `author(32)` + `d_len(1)` + `d(182)`
pub fn make_coordinate_index_key(coordinate: &Coordinate) -> Vec<u8> {
    let mut key: Vec<u8> = Vec::with_capacity(KIND_BE + PublicKey::LEN + 1 + TAG_VALUE_PAD_LEN);
    key.extend(coordinate.kind.as_u16().to_be_bytes());
    key.extend(coordinate.public_key.to_bytes());

    let dlen: usize = cmp::min(coordinate.identifier.len(), TAG_VALUE_PAD_LEN);
    key.push(dlen as u8);

    extend_key_with_tag_value(&mut key, dlen, &coordinate.identifier);

    key
}
