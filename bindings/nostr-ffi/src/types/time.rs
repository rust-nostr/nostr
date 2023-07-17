// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

pub struct Timestamp {
    inner: nostr::Timestamp,
}

impl From<nostr::Timestamp> for Timestamp {
    fn from(inner: nostr::Timestamp) -> Self {
        Self { inner }
    }
}

impl Timestamp {
    /// Get UNIX timestamp
    pub fn now() -> Self {
        Self {
            inner: nostr::Timestamp::now(),
        }
    }

    pub fn from_secs(secs: u64) -> Self {
        Self {
            inner: nostr::Timestamp::from(secs),
        }
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
