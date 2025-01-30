// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

#[cfg(not(target_arch = "wasm32"))]
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;

#[cfg(not(target_arch = "wasm32"))]
use tokio::sync::RwLock;

#[derive(Debug)]
#[cfg(not(target_arch = "wasm32"))]
pub(super) struct PingTracker {
    sent_at: RwLock<Instant>,
    last_nonce: AtomicU64,
    replied: AtomicBool,
}

#[derive(Debug, Default)]
#[cfg(target_arch = "wasm32")]
pub(super) struct PingTracker {}

#[cfg(not(target_arch = "wasm32"))]
impl Default for PingTracker {
    fn default() -> Self {
        Self {
            sent_at: RwLock::new(Instant::now()),
            last_nonce: AtomicU64::new(0),
            replied: AtomicBool::new(false),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl PingTracker {
    /// Get sent at
    #[inline]
    pub async fn sent_at(&self) -> Instant {
        *self.sent_at.read().await
    }

    /// Last nonce
    #[inline]
    pub fn last_nonce(&self) -> u64 {
        self.last_nonce.load(Ordering::SeqCst)
    }

    /// Replied
    #[inline]
    pub fn replied(&self) -> bool {
        self.replied.load(Ordering::SeqCst)
    }

    pub(super) async fn just_sent(&self) {
        let mut sent_at = self.sent_at.write().await;
        *sent_at = Instant::now();
    }

    #[inline]
    pub(super) fn set_last_nonce(&self, nonce: u64) {
        self.last_nonce.store(nonce, Ordering::SeqCst)
    }

    #[inline]
    pub(super) fn set_replied(&self, replied: bool) {
        self.replied.store(replied, Ordering::SeqCst);
    }
}
