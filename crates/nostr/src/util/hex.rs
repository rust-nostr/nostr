// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Hex

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

/// Hex error
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// An invalid character was found
    InvalidHexCharacter {
        /// Char
        c: char,
        /// Char index
        index: usize,
    },
    /// A hex string's length needs to be even, as two digits correspond to
    /// one byte.
    OddLength,
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidHexCharacter { c, index } => {
                write!(f, "Invalid character {} at position {}", c, index)
            }
            Self::OddLength => write!(f, "Odd number of digits"),
        }
    }
}

#[inline]
fn from_digit(num: u8) -> char {
    if num < 10 {
        (b'0' + num) as char
    } else {
        (b'a' + num - 10) as char
    }
}

/// Hex encode
pub fn encode<T>(data: T) -> String
where
    T: AsRef<[u8]>,
{
    let bytes: &[u8] = data.as_ref();
    let mut hex: String = String::with_capacity(2 * bytes.len());
    for byte in bytes.iter() {
        hex.push(from_digit(byte >> 4));
        hex.push(from_digit(byte & 0xF));
    }
    hex
}

const fn val(c: u8, idx: usize) -> Result<u8, Error> {
    match c {
        b'A'..=b'F' => Ok(c - b'A' + 10),
        b'a'..=b'f' => Ok(c - b'a' + 10),
        b'0'..=b'9' => Ok(c - b'0'),
        _ => Err(Error::InvalidHexCharacter {
            c: c as char,
            index: idx,
        }),
    }
}

/// Hex decode
pub fn decode<T>(hex: T) -> Result<Vec<u8>, Error>
where
    T: AsRef<[u8]>,
{
    let hex = hex.as_ref();
    let len = hex.len();

    if len % 2 != 0 {
        return Err(Error::OddLength);
    }

    let mut bytes: Vec<u8> = Vec::with_capacity(len / 2);

    for i in (0..len).step_by(2) {
        let high = val(hex[i], i)?;
        let low = val(hex[i + 1], i + 1)?;
        bytes.push(high << 4 | low);
    }

    Ok(bytes)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_encode() {
        assert_eq!(encode("foobar"), "666f6f626172");
    }

    #[test]
    fn test_decode() {
        assert_eq!(
            decode("666f6f626172"),
            Ok(String::from("foobar").into_bytes())
        );
    }

    #[test]
    pub fn test_invalid_length() {
        assert_eq!(decode("1").unwrap_err(), Error::OddLength);
        assert_eq!(decode("666f6f6261721").unwrap_err(), Error::OddLength);
    }

    #[test]
    pub fn test_invalid_char() {
        assert_eq!(
            decode("66ag").unwrap_err(),
            Error::InvalidHexCharacter { c: 'g', index: 3 }
        );
    }
}

#[cfg(bench)]
mod benches {
    use super::*;
    use crate::test::{black_box, Bencher};

    #[bench]
    pub fn hex_encode(bh: &mut Bencher) {
        bh.iter(|| {
            black_box(encode("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"));
        });
    }
}
