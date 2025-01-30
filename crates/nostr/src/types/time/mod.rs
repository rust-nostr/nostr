// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Time

use alloc::string::{String, ToString};
use core::fmt;
use core::ops::{Add, Range, Sub};
use core::str::{self, FromStr};
use core::time::Duration;

#[cfg(feature = "std")]
use secp256k1::rand::rngs::OsRng;
use secp256k1::rand::Rng;

mod supplier;

pub use self::supplier::TimeSupplier;
#[cfg(feature = "std")]
pub use self::supplier::{Instant, SystemTime, UNIX_EPOCH};

// 2000-03-01 (mod 400 year, immediately after feb29)
const LEAPOCH: i64 = 11017;
const DAYS_PER_400Y: i64 = 365 * 400 + 97;
const DAYS_PER_100Y: i64 = 365 * 100 + 24;
const DAYS_PER_4Y: i64 = 365 * 4 + 1;

const TO_HUMAN_DATE_BUF: [u8; 20] = [
    b'0', b'0', b'0', b'0', b'-', b'0', b'0', b'-', b'0', b'0', b'T', b'0', b'0', b':', b'0', b'0',
    b':', b'0', b'0', b'Z',
];

/// Unix timestamp in seconds
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Timestamp(u64);

impl Timestamp {
    /// Construct from seconds
    #[inline]
    pub const fn from_secs(secs: u64) -> Self {
        Self(secs)
    }

    /// Compose `0` timestamp
    #[inline]
    pub const fn zero() -> Self {
        Self::from_secs(0)
    }

    /// The minimum representable timestamp
    #[inline]
    pub const fn min() -> Self {
        Self::from_secs(u64::MIN)
    }

    /// The maximum representable timestamp
    #[inline]
    pub const fn max() -> Self {
        Self::from_secs(u64::MAX)
    }

    /// Get UNIX timestamp
    #[cfg(feature = "std")]
    pub fn now() -> Self {
        let ts: u64 = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Self::from_secs(ts)
    }

    /// Get UNIX timestamp from a specified [`TimeSupplier`]
    pub fn now_with_supplier<T>(supplier: &T) -> Self
    where
        T: TimeSupplier,
    {
        let now = supplier.now();
        let starting_point = supplier.starting_point();
        let duration = supplier.elapsed_since(now, starting_point);
        supplier.to_timestamp(duration)
    }

    /// Get tweaked UNIX timestamp
    ///
    /// Remove a random number of seconds from now
    #[cfg(feature = "std")]
    pub fn tweaked(range: Range<u64>) -> Self {
        let mut now: Timestamp = Self::now();
        now.tweak(range);
        now
    }

    /// Get tweaked UNIX timestamp
    ///
    /// Remove a random number of seconds from now
    pub fn tweaked_with_supplier_and_rng<T, R>(supplier: &T, rng: &mut R, range: Range<u64>) -> Self
    where
        T: TimeSupplier,
        R: Rng,
    {
        let mut now: Timestamp = Self::now_with_supplier(supplier);
        now.tweak_with_rng(rng, range);
        now
    }

    /// Remove a random number of seconds from [`Timestamp`]
    #[inline]
    #[cfg(feature = "std")]
    pub fn tweak(&mut self, range: Range<u64>) {
        self.tweak_with_rng(&mut OsRng, range);
    }

    /// Remove a random number of seconds from [`Timestamp`]
    pub fn tweak_with_rng<R>(&mut self, rng: &mut R, range: Range<u64>)
    where
        R: Rng,
    {
        let secs: u64 = rng.gen_range(range);
        self.0 = self.0.saturating_sub(secs);
    }

    /// Get timestamp as [`u64`]
    #[inline]
    pub fn as_u64(&self) -> u64 {
        self.0
    }

    /// Check if timestamp is `0`
    #[inline]
    pub fn is_zero(&self) -> bool {
        self.0 == 0
    }

    /// Convert [`Timestamp`] to human datetime
    pub fn to_human_datetime(&self) -> String {
        let timestamp: u64 = self.as_u64();

        if timestamp >= 253_402_300_800 {
            // Year 9999
            return String::from("Unavailable");
        }

        let days = (timestamp / 86400) as i64 - LEAPOCH;
        let secs_of_day = timestamp % 86400;

        let mut qc_cycles = days / DAYS_PER_400Y;
        let mut remdays = days % DAYS_PER_400Y;

        if remdays < 0 {
            remdays += DAYS_PER_400Y;
            qc_cycles -= 1;
        }

        let mut c_cycles = remdays / DAYS_PER_100Y;
        if c_cycles == 4 {
            c_cycles -= 1;
        }
        remdays -= c_cycles * DAYS_PER_100Y;

        let mut q_cycles = remdays / DAYS_PER_4Y;
        if q_cycles == 25 {
            q_cycles -= 1;
        }
        remdays -= q_cycles * DAYS_PER_4Y;

        let mut remyears = remdays / 365;
        if remyears == 4 {
            remyears -= 1;
        }
        remdays -= remyears * 365;

        let mut year = 2000 + remyears + 4 * q_cycles + 100 * c_cycles + 400 * qc_cycles;

        let months = [31, 30, 31, 30, 31, 31, 30, 31, 30, 31, 31, 29];
        let mut mon = 0;
        for mon_len in months.iter() {
            mon += 1;
            if remdays < *mon_len {
                break;
            }
            remdays -= *mon_len;
        }
        let mday = remdays + 1;
        let mon = if mon + 2 > 12 {
            year += 1;
            mon - 10
        } else {
            mon + 2
        };

        let mut buf: [u8; 20] = TO_HUMAN_DATE_BUF;

        buf[0] = b'0' + (year / 1000) as u8;
        buf[1] = b'0' + (year / 100 % 10) as u8;
        buf[2] = b'0' + (year / 10 % 10) as u8;
        buf[3] = b'0' + (year % 10) as u8;
        buf[5] = b'0' + (mon / 10) as u8;
        buf[6] = b'0' + (mon % 10) as u8;
        buf[8] = b'0' + (mday / 10) as u8;
        buf[9] = b'0' + (mday % 10) as u8;
        buf[11] = b'0' + (secs_of_day / 3600 / 10) as u8;
        buf[12] = b'0' + (secs_of_day / 3600 % 10) as u8;
        buf[14] = b'0' + (secs_of_day / 60 / 10 % 6) as u8;
        buf[15] = b'0' + (secs_of_day / 60 % 10) as u8;
        buf[17] = b'0' + (secs_of_day / 10 % 6) as u8;
        buf[18] = b'0' + (secs_of_day % 10) as u8;

        str::from_utf8(&buf).unwrap_or_default().to_string()
    }
}

impl Default for Timestamp {
    #[inline]
    fn default() -> Self {
        Self::zero()
    }
}

impl From<u64> for Timestamp {
    fn from(secs: u64) -> Self {
        Self::from_secs(secs)
    }
}

impl FromStr for Timestamp {
    type Err = core::num::ParseIntError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from_secs(s.parse::<u64>()?))
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Add<Timestamp> for Timestamp {
    type Output = Self;
    fn add(self, rhs: Timestamp) -> Self::Output {
        Self::from_secs(self.0.saturating_add(rhs.as_u64()))
    }
}

impl Sub<Timestamp> for Timestamp {
    type Output = Self;
    fn sub(self, rhs: Timestamp) -> Self::Output {
        Self::from_secs(self.0.saturating_sub(rhs.as_u64()))
    }
}

impl Add<Duration> for Timestamp {
    type Output = Self;

    fn add(self, rhs: Duration) -> Self::Output {
        Self::from_secs(self.0.saturating_add(rhs.as_secs()))
    }
}

impl Sub<Duration> for Timestamp {
    type Output = Self;

    fn sub(self, rhs: Duration) -> Self::Output {
        Self::from_secs(self.0.saturating_sub(rhs.as_secs()))
    }
}

impl Add<u64> for Timestamp {
    type Output = Self;

    fn add(self, rhs: u64) -> Self::Output {
        Self::from_secs(self.0.saturating_add(rhs))
    }
}

impl Sub<u64> for Timestamp {
    type Output = Self;

    fn sub(self, rhs: u64) -> Self::Output {
        Self::from_secs(self.0.saturating_sub(rhs))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timestamp_to_human_datetime() {
        let timestamp = Timestamp::from(1682060685);
        assert_eq!(
            timestamp.to_human_datetime(),
            String::from("2023-04-21T07:04:45Z")
        );
    }
}

#[cfg(bench)]
mod benches {
    use test::{black_box, Bencher};

    use super::*;

    #[bench]
    pub fn timestamp_to_human_datetime(bh: &mut Bencher) {
        let timestamp = Timestamp::from(1682060685);
        bh.iter(|| {
            black_box(timestamp.to_human_datetime());
        });
    }
}
