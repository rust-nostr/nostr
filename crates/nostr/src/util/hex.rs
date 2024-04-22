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
    /// A hex string's length needs to be even, as two digits correspond to one byte.
    OddLength,
    /// Invalid len
    InvalidLength,
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
            Self::InvalidLength => write!(f, "Invalid length"),
        }
    }
}

#[inline]
const fn from_digit(num: u8) -> u8 {
    if num < 10 {
        b'0' + num
    } else {
        b'a' + num - 10
    }
}

/// Hex encode
#[inline]
pub fn encode<T>(data: T) -> String
where
    T: AsRef<[u8]>,
{
    let bytes: &[u8] = data.as_ref();
    let mut hex: String = String::with_capacity(2 * bytes.len());
    for byte in bytes.iter() {
        hex.push(from_digit(byte >> 4) as char);
        hex.push(from_digit(byte & 0xF) as char);
    }
    hex
}

/// Hex decode
#[inline]
pub fn decode<T>(hex: T) -> Result<Vec<u8>, Error>
where
    T: AsRef<[u8]>,
{
    let hex: &[u8] = hex.as_ref();
    let len: usize = hex.len();

    let mut bytes: Vec<u8> = vec![0u8; len / 2];
    decode_to_slice(hex, &mut bytes)?;
    Ok(bytes)
}

/// Hex decode to slice
#[inline]
pub fn decode_to_slice<T>(hex: T, out: &mut [u8]) -> Result<(), Error>
where
    T: AsRef<[u8]>,
{
    let hex: &[u8] = hex.as_ref();
    let hex_len: usize = hex.len();

    if hex_len % 2 != 0 {
        return Err(Error::OddLength);
    }

    if hex_len / 2 != out.len() {
        return Err(Error::InvalidLength);
    }

    for (i, byte) in out.iter_mut().enumerate() {
        let high_idx: usize = i * 2;
        let high: u8 = val(hex[high_idx], high_idx)?;

        let low_idx: usize = high_idx + 1;
        let low: u8 = val(hex[low_idx], low_idx)?;

        *byte = high << 4 | low;
    }

    Ok(())
}

#[inline]
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

#[cfg(test)]
mod tests {
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

    const EVENT_JSON: &str = r#"{"content":"uRuvYr585B80L6rSJiHocw==?iv=oh6LVqdsYYol3JfFnXTbPA==","created_at":1640839235,"id":"2be17aa3031bdcb006f0fce80c146dea9c1c0268b0af2398bb673365c6444d45","kind":4,"pubkey":"f86c44a2de95d9149b51c6a29afeabba264c18e2fa7c49de93424a0c56947785","sig":"a5d9290ef9659083c490b303eb7ee41356d8778ff19f2f91776c8dc4443388a64ffcf336e61af4c25c05ac3ae952d1ced889ed655b67790891222aaa15b99fdd","tags":[["p","13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"]]}"#;

    #[bench]
    pub fn hex_encode(bh: &mut Bencher) {
        bh.iter(|| {
            black_box(encode(EVENT_JSON));
        });
    }

    #[bench]
    pub fn hex_decode(bh: &mut Bencher) {
        let h = "7b22636f6e74656e74223a227552757659723538354238304c3672534a69486f63773d3d3f69763d6f68364c5671647359596f6c334a66466e58546250413d3d222c22637265617465645f6174223a313634303833393233352c226964223a2232626531376161333033316264636230303666306663653830633134366465613963316330323638623061663233393862623637333336356336343434643435222c226b696e64223a342c227075626b6579223a2266383663343461326465393564393134396235316336613239616665616262613236346331386532666137633439646539333432346130633536393437373835222c22736967223a226135643932393065663936353930383363343930623330336562376565343133353664383737386666313966326639313737366338646334343433333838613634666663663333366536316166346332356330356163336165393532643163656438383965643635356236373739303839313232326161613135623939666464222c2274616773223a5b5b2270222c2231336164633531316465376531636663663163366237663633363566623561303334343264376263616366353635656135376661373737303931326330323364225d5d7d";
        bh.iter(|| {
            black_box(decode(h)).unwrap();
        });
    }
}
