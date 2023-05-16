// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Time supplier

use core::ops::Sub;
use core::time::Duration;
#[cfg(all(feature = "std", not(target_arch = "wasm32")))]
use std::time::Instant;
#[cfg(feature = "std")]
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(target_arch = "wasm32")]
use instant::Instant as InstantWasm32;

use super::Timestamp;

/// Helper trait for acquiring time in `no_std` environments.
pub trait TimeSupplier {
    /// The current time from the specified `TimeSupplier`
    type Now: Clone + Sub;
    /// The starting point for the specified `TimeSupplier`
    type StartingPoint: Clone;

    /// Get the current time as the associated `StartingPoint` type
    fn now(&self) -> Self::StartingPoint;

    /// Get the current time as the associated `Now` type
    fn instant_now(&self) -> Self::Now;

    /// Get the starting point from the specified `TimeSupplier`
    fn starting_point(&self) -> Self::StartingPoint;

    /// Get a duration since the StartingPoint.
    fn duration_since_starting_point(&self, now: Self::StartingPoint) -> Duration;

    /// Get the elapsed time as `Duration` starting from `since` to `now`
    fn elapsed_instant_since(&self, now: Self::Now, since: Self::Now) -> Duration;

    /// Get the elapsed time as `Duration` starting from `since` to `now`
    fn elapsed_since(&self, now: Self::StartingPoint, since: Self::StartingPoint) -> Duration;

    /// Convert the specified `Duration` to `Timestamp`
    fn to_timestamp(&self, duration: Duration) -> Timestamp {
        Timestamp::from(duration.as_secs())
    }
}

#[cfg(all(feature = "std", target_arch = "wasm32"))]
impl TimeSupplier for InstantWasm32 {
    type Now = InstantWasm32;
    type StartingPoint = std::time::SystemTime;

    fn now(&self) -> Self::StartingPoint {
        SystemTime::now()
    }

    fn instant_now(&self) -> Self::Now {
        InstantWasm32::now()
    }

    fn starting_point(&self) -> Self::StartingPoint {
        UNIX_EPOCH
    }

    fn duration_since_starting_point(&self, now: Self::StartingPoint) -> Duration {
        now.duration_since(self.starting_point())
            .unwrap_or_default()
    }

    fn elapsed_instant_since(&self, now: Self::Now, since: Self::Now) -> Duration {
        now - since
    }

    fn elapsed_since(&self, now: Self::StartingPoint, since: Self::StartingPoint) -> Duration {
        now.duration_since(since).unwrap_or_default()
    }
}

#[cfg(all(feature = "std", not(target_arch = "wasm32")))]
impl TimeSupplier for Instant {
    type Now = Instant;
    type StartingPoint = std::time::SystemTime;

    fn now(&self) -> Self::StartingPoint {
        SystemTime::now()
    }

    fn instant_now(&self) -> Self::Now {
        Instant::now()
    }

    fn starting_point(&self) -> Self::StartingPoint {
        UNIX_EPOCH
    }

    fn duration_since_starting_point(&self, now: Self::StartingPoint) -> Duration {
        now.duration_since(self.starting_point())
            .unwrap_or_default()
    }

    fn elapsed_instant_since(&self, now: Self::Now, since: Self::Now) -> Duration {
        now - since
    }

    fn elapsed_since(&self, now: Self::StartingPoint, since: Self::StartingPoint) -> Duration {
        now.duration_since(since).unwrap_or_default()
    }
}
