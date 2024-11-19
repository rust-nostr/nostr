// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::hash::Hash;
use std::num::NonZeroUsize;

use lru::LruCache;

pub mod events;
pub mod tree;

pub(crate) fn new_lru_cache<K, V>(size: Option<usize>) -> LruCache<K, V>
where
    K: Hash + Eq,
{
    match size {
        Some(size) => match NonZeroUsize::new(size) {
            Some(size) => LruCache::new(size),
            None => LruCache::unbounded(),
        },
        None => LruCache::unbounded(),
    }
}
