// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;
use std::sync::Arc;

use uniffi::Object;

#[derive(Object)]
pub struct Timestamp {
    inner: nostr::Timestamp,
}

impl From<nostr::Timestamp> for Timestamp {
    fn from(inner: nostr::Timestamp) -> Self {
        Self { inner }
    }
}

impl Deref for Timestamp {
    type Target = nostr::Timestamp;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[uniffi::export]
impl Timestamp {
    /// Get UNIX timestamp
    #[uniffi::constructor]
    pub fn now() -> Arc<Self> {
        Arc::new(Self {
            inner: nostr::Timestamp::now(),
        })
    }

    #[uniffi::constructor]
    pub fn from_secs(secs: u64) -> Arc<Self> {
        Arc::new(Self {
            inner: nostr::Timestamp::from(secs),
        })
    }

    /// Get timestamp as [`u64`]
    pub fn as_secs(&self) -> u64 {
        self.inner.as_u64()
    }

    /// Convert [`Timestamp`] to human datetime
    pub fn to_human_datetime(&self) -> String {
        self.inner.to_human_datetime()
    }
}
