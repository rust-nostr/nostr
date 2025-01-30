// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Image

use core::fmt;
use core::num::ParseIntError;
use core::str::{FromStr, Split};

#[derive(Debug, PartialEq, Eq)]
/// Image error
pub enum Error {
    /// Impossible to parse integer
    ParseIntError(ParseIntError),
    /// Invalid Image Dimensions
    InvalidImageDimensions,
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ParseIntError(e) => write!(f, "{e}"),
            Self::InvalidImageDimensions => write!(f, "Invalid image dimensions"),
        }
    }
}

impl From<ParseIntError> for Error {
    fn from(e: ParseIntError) -> Self {
        Self::ParseIntError(e)
    }
}

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

impl FromStr for ImageDimensions {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut spitted: Split<char> = s.split('x');
        if let (Some(width), Some(height)) = (spitted.next(), spitted.next()) {
            Ok(Self::new(width.parse()?, height.parse()?))
        } else {
            Err(Error::InvalidImageDimensions)
        }
    }
}

impl fmt::Display for ImageDimensions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}x{}", self.width, self.height)
    }
}
