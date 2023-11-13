// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! RocksDB Custom Operators

use std::collections::HashSet;

use nostr::Url;
use nostr_database::{FlatBufferBuilder, FlatBufferDecode, FlatBufferEncode};
use rocksdb::MergeOperands;

pub(crate) fn relay_urls_merge_operator(
    _new_key: &[u8],
    existing: Option<&[u8]>,
    operands: &MergeOperands,
) -> Option<Vec<u8>> {
    let mut existing: HashSet<Url> = match existing {
        Some(val) => HashSet::decode(val).ok()?,
        None => HashSet::with_capacity(operands.len()),
    };

    for operand in operands.into_iter() {
        existing.extend(HashSet::decode(operand).ok()?);
    }

    let mut fbb = FlatBufferBuilder::with_capacity(existing.len() * 32 * 2); // Check capacity size if correct
    Some(existing.encode(&mut fbb).to_vec())
}
