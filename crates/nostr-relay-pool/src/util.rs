// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::sync::Arc;

use tokio::sync::Mutex;

/// Take ownership of `T` from `Arc<Mutex<T>>`.
///
/// Try to take ownership of result without clone.
/// Clone if fail to unwrap inner value of `Arc`.
pub(crate) async fn take_mutex_ownership<T>(val: Arc<Mutex<T>>) -> T
where
    T: Clone,
{
    match Arc::try_unwrap(val) {
        Ok(mutex) => mutex.into_inner(),
        Err(arc) => {
            let lock = arc.lock().await;
            lock.clone()
        }
    }
}
