// Copyright (c) 2021 Paul Miller
// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP13: Proof of Work
//!
//! <https://github.com/nostr-protocol/nips/blob/master/13.md>

use alloc::string::String;
use alloc::vec::Vec;
use core::any::Any;
use core::fmt::Debug;
use core::num::NonZeroU8;

#[cfg(feature = "std")]
mod blocking_wrapper;
#[cfg(feature = "pow-multi-thread")]
mod multi_thread;
mod single_thread;

#[cfg(feature = "pow-multi-thread")]
pub use self::multi_thread::*;
pub use self::single_thread::*;
use crate::UnsignedEvent;
use crate::util::BoxedFuture;

/// Gets the number of leading zero bits. Result is between 0 and 255.
#[inline]
pub fn get_leading_zero_bits<T>(h: T) -> u8
where
    T: AsRef<[u8]>,
{
    let mut res: u8 = 0u8;
    for b in h.as_ref().iter() {
        if *b == 0 {
            res += 8;
        } else {
            res += b.leading_zeros() as u8;
            return res;
        }
    }
    res
}

/// Returns all possible ID prefixes (hex) that have the specified number of leading zero bits.
///
/// Possible values: 0-255
pub fn get_prefixes_for_difficulty(leading_zero_bits: u8) -> Vec<String> {
    let mut r = Vec::new();

    if leading_zero_bits == 0 {
        return r;
    }

    // Up to 64
    let prefix_hex_len = if leading_zero_bits % 4 == 0 {
        leading_zero_bits / 4
    } else {
        leading_zero_bits / 4 + 1
    };

    // Bitlength of relevant prefixes, from 4 (prefix has at least 1 hex char) to 256 (all 64 chars)
    let prefix_bits: u16 = prefix_hex_len as u16 * 4;

    // pow expects u32
    for i in 0..2_u8.pow((prefix_bits - leading_zero_bits as u16) as u32) {
        let p = format!("{:01$x}", i, prefix_hex_len as usize); // https://stackoverflow.com/a/26286238
        r.push(p);
    }

    r
}

/// A trait for custom Proof of Work computation.
pub trait PowAdapter: Any + Debug {
    /// Error
    type Error;

    /// Computes Proof of Work for an unsigned event to meet the target
    /// difficulty.
    fn compute(
        &self,
        unsigned: UnsignedEvent,
        difficulty: NonZeroU8,
    ) -> Result<UnsignedEvent, Self::Error>;
}

/// A trait for custom Proof of Work computation.
pub trait AsyncPowAdapter: Any + Debug + Send + Sync {
    /// Error
    type Error;

    /// Computes Proof of Work for an unsigned event to meet the target
    /// difficulty.
    fn compute(
        &self,
        unsigned: UnsignedEvent,
        difficulty: NonZeroU8,
    ) -> BoxedFuture<'_, Result<UnsignedEvent, Self::Error>>;
}

#[cfg(test)]
pub mod tests {
    #[cfg(feature = "std")]
    use core::convert::Infallible;
    use core::str::FromStr;
    #[cfg(feature = "std")]
    use core::time::Duration;

    use hashes::sha256::Hash as Sha256Hash;

    use super::*;
    #[cfg(feature = "std")]
    use crate::{EventBuilder, PublicKey, Tag, TagKind};

    #[test]
    fn check_get_leading_zeroes() {
        assert_eq!(
            4,
            get_leading_zero_bits(
                Sha256Hash::from_str(
                    "0fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
                )
                .unwrap()
            )
        );
        assert_eq!(
            3,
            get_leading_zero_bits(
                Sha256Hash::from_str(
                    "1fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
                )
                .unwrap()
            )
        );
        assert_eq!(
            2,
            get_leading_zero_bits(
                Sha256Hash::from_str(
                    "2fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
                )
                .unwrap()
            )
        );
        assert_eq!(
            2,
            get_leading_zero_bits(
                Sha256Hash::from_str(
                    "3fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
                )
                .unwrap()
            )
        );
        assert_eq!(
            1,
            get_leading_zero_bits(
                Sha256Hash::from_str(
                    "4fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
                )
                .unwrap()
            )
        );
        assert_eq!(
            1,
            get_leading_zero_bits(
                Sha256Hash::from_str(
                    "5fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
                )
                .unwrap()
            )
        );
        assert_eq!(
            1,
            get_leading_zero_bits(
                Sha256Hash::from_str(
                    "6fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
                )
                .unwrap()
            )
        );
        assert_eq!(
            1,
            get_leading_zero_bits(
                Sha256Hash::from_str(
                    "7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
                )
                .unwrap()
            )
        );

        assert_eq!(
            0,
            get_leading_zero_bits(
                Sha256Hash::from_str(
                    "8fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
                )
                .unwrap()
            )
        );
        assert_eq!(
            0,
            get_leading_zero_bits(
                Sha256Hash::from_str(
                    "9fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
                )
                .unwrap()
            )
        );
        assert_eq!(
            0,
            get_leading_zero_bits(
                Sha256Hash::from_str(
                    "afffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
                )
                .unwrap()
            )
        );
        assert_eq!(
            0,
            get_leading_zero_bits(
                Sha256Hash::from_str(
                    "bfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
                )
                .unwrap()
            )
        );
        assert_eq!(
            0,
            get_leading_zero_bits(
                Sha256Hash::from_str(
                    "cfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
                )
                .unwrap()
            )
        );
        assert_eq!(
            0,
            get_leading_zero_bits(
                Sha256Hash::from_str(
                    "dfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
                )
                .unwrap()
            )
        );
        assert_eq!(
            0,
            get_leading_zero_bits(
                Sha256Hash::from_str(
                    "efffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
                )
                .unwrap()
            )
        );
        assert_eq!(
            0,
            get_leading_zero_bits(
                Sha256Hash::from_str(
                    "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
                )
                .unwrap()
            )
        );

        assert_eq!(
            2,
            get_leading_zero_bits(
                Sha256Hash::from_str(
                    "20ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
                )
                .unwrap()
            )
        );
        assert_eq!(
            2,
            get_leading_zero_bits(
                Sha256Hash::from_str(
                    "21ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
                )
                .unwrap()
            )
        );
        assert_eq!(
            2,
            get_leading_zero_bits(
                Sha256Hash::from_str(
                    "22ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
                )
                .unwrap()
            )
        );
        assert_eq!(
            2,
            get_leading_zero_bits(
                Sha256Hash::from_str(
                    "23ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
                )
                .unwrap()
            )
        );
        assert_eq!(
            2,
            get_leading_zero_bits(
                Sha256Hash::from_str(
                    "24ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
                )
                .unwrap()
            )
        );
        assert_eq!(
            2,
            get_leading_zero_bits(
                Sha256Hash::from_str(
                    "25ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
                )
                .unwrap()
            )
        );
        assert_eq!(
            2,
            get_leading_zero_bits(
                Sha256Hash::from_str(
                    "26ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
                )
                .unwrap()
            )
        );
        assert_eq!(
            2,
            get_leading_zero_bits(
                Sha256Hash::from_str(
                    "27ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
                )
                .unwrap()
            )
        );
        assert_eq!(
            2,
            get_leading_zero_bits(
                Sha256Hash::from_str(
                    "28ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
                )
                .unwrap()
            )
        );
        assert_eq!(
            2,
            get_leading_zero_bits(
                Sha256Hash::from_str(
                    "29ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
                )
                .unwrap()
            )
        );
        assert_eq!(
            2,
            get_leading_zero_bits(
                Sha256Hash::from_str(
                    "2affffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
                )
                .unwrap()
            )
        );
        assert_eq!(
            2,
            get_leading_zero_bits(
                Sha256Hash::from_str(
                    "2bffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
                )
                .unwrap()
            )
        );
        assert_eq!(
            2,
            get_leading_zero_bits(
                Sha256Hash::from_str(
                    "2cffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
                )
                .unwrap()
            )
        );
        assert_eq!(
            2,
            get_leading_zero_bits(
                Sha256Hash::from_str(
                    "2dffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
                )
                .unwrap()
            )
        );
        assert_eq!(
            2,
            get_leading_zero_bits(
                Sha256Hash::from_str(
                    "2effffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
                )
                .unwrap()
            )
        );
        assert_eq!(
            2,
            get_leading_zero_bits(
                Sha256Hash::from_str(
                    "2fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
                )
                .unwrap()
            )
        );

        assert_eq!(
            248,
            get_leading_zero_bits(
                Sha256Hash::from_str(
                    "00000000000000000000000000000000000000000000000000000000000000ff"
                )
                .unwrap()
            )
        );
        assert_eq!(
            252,
            get_leading_zero_bits(
                Sha256Hash::from_str(
                    "000000000000000000000000000000000000000000000000000000000000000f"
                )
                .unwrap()
            )
        );
        assert_eq!(
            255,
            get_leading_zero_bits(
                Sha256Hash::from_str(
                    "0000000000000000000000000000000000000000000000000000000000000001"
                )
                .unwrap()
            )
        );
    }

    #[test]
    fn check_find_prefixes_for_pow() {
        assert!(get_prefixes_for_difficulty(0).is_empty());

        assert_eq!(
            get_prefixes_for_difficulty(1),
            vec!["0", "1", "2", "3", "4", "5", "6", "7"]
        );
        assert_eq!(get_prefixes_for_difficulty(2), vec!["0", "1", "2", "3"]);
        assert_eq!(get_prefixes_for_difficulty(3), vec!["0", "1"]);
        assert_eq!(get_prefixes_for_difficulty(4), vec!["0"]);

        assert_eq!(
            get_prefixes_for_difficulty(5),
            vec!["00", "01", "02", "03", "04", "05", "06", "07"]
        );
        assert_eq!(get_prefixes_for_difficulty(6), vec!["00", "01", "02", "03"]);
        assert_eq!(get_prefixes_for_difficulty(7), vec!["00", "01"]);
        assert_eq!(get_prefixes_for_difficulty(8), vec!["00"]);

        assert_eq!(
            get_prefixes_for_difficulty(9),
            vec!["000", "001", "002", "003", "004", "005", "006", "007"]
        );
        assert_eq!(
            get_prefixes_for_difficulty(10),
            vec!["000", "001", "002", "003"]
        );
        assert_eq!(get_prefixes_for_difficulty(11), vec!["000", "001"]);
        assert_eq!(get_prefixes_for_difficulty(12), vec!["000"]);

        assert_eq!(
            get_prefixes_for_difficulty(254),
            vec![
                "0000000000000000000000000000000000000000000000000000000000000000",
                "0000000000000000000000000000000000000000000000000000000000000001",
                "0000000000000000000000000000000000000000000000000000000000000002",
                "0000000000000000000000000000000000000000000000000000000000000003"
            ]
        );
        assert_eq!(
            get_prefixes_for_difficulty(255),
            vec![
                "0000000000000000000000000000000000000000000000000000000000000000",
                "0000000000000000000000000000000000000000000000000000000000000001"
            ]
        );
    }

    #[test]
    #[cfg(feature = "std")]
    fn custom_adapter() {
        #[derive(Debug)]
        struct TestAdapter;

        impl PowAdapter for TestAdapter {
            type Error = Infallible;

            fn compute(
                &self,
                mut unsigned_event: UnsignedEvent,
                target_difficulty: NonZeroU8,
            ) -> Result<UnsignedEvent, Self::Error> {
                unsigned_event
                    .tags
                    .push(Tag::pow(3490, target_difficulty.get()));
                unsigned_event.ensure_id();
                Ok(unsigned_event)
            }
        }

        let unsigned = EventBuilder::text_note(
            "Why must I find leading zero bits? Is there no beauty in the ones?",
        )
        .build(PublicKey::from_slice(&[0; 32]).unwrap());

        let unsigned = unsigned
            .mine(&TestAdapter, NonZeroU8::new(2).unwrap())
            .unwrap();

        let Some(nonce_tag) = unsigned.tags.find(TagKind::Nonce) else {
            panic!("nonce tag should be exist")
        };

        assert_eq!(nonce_tag.as_slice()[1], "3490");
        assert_eq!(nonce_tag.as_slice()[2], "2");
    }

    #[tokio::test]
    #[cfg(feature = "std")]
    async fn custom_async_adapter() {
        #[derive(Debug)]
        struct AsyncTestAdapter;

        impl AsyncPowAdapter for AsyncTestAdapter {
            type Error = Infallible;

            fn compute(
                &self,
                mut unsigned_event: UnsignedEvent,
                target_difficulty: NonZeroU8,
            ) -> BoxedFuture<'_, Result<UnsignedEvent, Self::Error>> {
                Box::pin(async move {
                    // Simulate an async work
                    tokio::time::sleep(Duration::from_secs(2)).await;

                    unsigned_event
                        .tags
                        .push(Tag::pow(3490, target_difficulty.get()));
                    unsigned_event.ensure_id();
                    Ok(unsigned_event)
                })
            }
        }

        let unsigned = EventBuilder::text_note(
            "Why must I find leading zero bits? Is there no beauty in the ones?",
        )
        .build(PublicKey::from_slice(&[0; 32]).unwrap());

        let unsigned = unsigned
            .mine_async(&AsyncTestAdapter, NonZeroU8::new(2).unwrap())
            .await
            .unwrap();

        let Some(nonce_tag) = unsigned.tags.find(TagKind::Nonce) else {
            panic!("nonce tag should be exist")
        };

        assert_eq!(nonce_tag.as_slice()[1], "3490");
        assert_eq!(nonce_tag.as_slice()[2], "2");
    }
}
