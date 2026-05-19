// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Image

use core::fmt;
use core::str::{FromStr, Split};

use crate::error::{Error, ErrorKind};

/// Simple struct to hold `width` x `height`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ImageDimensions {
    /// Width
    pub width: u64,
    /// Height
    pub height: u64,
}

impl ImageDimensions {
    /// Net image dimensions
    #[inline]
    pub fn new(width: u64, height: u64) -> Self {
        Self { width, height }
    }
}

impl fmt::Display for ImageDimensions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}x{}", self.width, self.height)
    }
}

impl FromStr for ImageDimensions {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut spitted: Split<char> = s.split('x');
        if let (Some(width), Some(height)) = (spitted.next(), spitted.next()) {
            Ok(Self::new(
                width.parse().map_err(Error::malformed)?,
                height.parse().map_err(Error::malformed)?,
            ))
        } else {
            Err(Error::with_static_message(
                ErrorKind::Invalid,
                "invalid dimensions",
            ))
        }
    }
}
