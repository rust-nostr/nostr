// Copyright (c) 2021 Paul Miller
// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP13: Proof of Work
//!
//! <https://github.com/nostr-protocol/nips/blob/master/13.md>

use alloc::boxed::Box;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::fmt;
#[cfg(feature = "pow-multi-thread")]
use std::{
    sync::atomic::{AtomicBool, Ordering},
    thread::{self, JoinHandle},
};

use crate::util::BoxedFuture;
use crate::{Tag, Timestamp, UnsignedEvent};

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
pub trait PowAdapter: fmt::Debug + Send + Sync {
    /// Computes Proof of Work for an unsigned event to meet the target
    /// difficulty.
    fn compute(
        &self,
        unsigned_event: UnsignedEvent,
        target_difficulty: u8,
        custom_created_at: Option<Timestamp>,
    ) -> BoxedFuture<'_, UnsignedEvent>;
}

impl<T> PowAdapter for Arc<T>
where
    T: PowAdapter,
{
    fn compute(
        &self,
        unsigned_event: UnsignedEvent,
        target_difficulty: u8,
        custom_created_at: Option<Timestamp>,
    ) -> BoxedFuture<'_, UnsignedEvent> {
        self.as_ref()
            .compute(unsigned_event, target_difficulty, custom_created_at)
    }
}

/// A single-threaded PoW miner implementation
#[derive(Debug)]
pub struct SingleThreadPow;

impl PowAdapter for SingleThreadPow {
    fn compute(
        &self,
        mut unsigned_event: UnsignedEvent,
        target_difficulty: u8,
        custom_created_at: Option<Timestamp>,
    ) -> BoxedFuture<'_, UnsignedEvent> {
        Box::pin(async move {
            let mut nonce: u128 = 0;
            unsigned_event.tags.push(Tag::pow(0, target_difficulty));
            let pow_tag_index = unsigned_event.tags.len() - 1;

            loop {
                nonce += 1;

                if nonce % 1024 == 0 {
                    unsigned_event.created_at = custom_created_at.unwrap_or_else(Timestamp::now);
                    // TODO: yield to not block the async runtime
                    // REF: https://github.com/rust-nostr/nostr/issues/921
                }

                unsigned_event.tags[pow_tag_index] = Tag::pow(nonce, target_difficulty);

                let event_id = unsigned_event.compute_id();
                if get_leading_zero_bits(event_id.as_bytes()) == target_difficulty {
                    unsigned_event.id = Some(event_id);
                    return unsigned_event;
                }
            }
        })
    }
}

/// A multi-threaded Proof-of-Work miner.
///
/// Fallback to [`SingleThreadPow`] if:
/// - the number of threads is `1`;
/// - thread spawning or coordination fails;
/// - no valid solution is found by any thread (rare edge case)
#[derive(Debug)]
#[cfg(feature = "pow-multi-thread")]
pub struct MultiThreadPow;

#[cfg(feature = "pow-multi-thread")]
impl PowAdapter for MultiThreadPow {
    fn compute(
        &self,
        unsigned_event: UnsignedEvent,
        target_difficulty: u8,
        custom_created_at: Option<Timestamp>,
    ) -> BoxedFuture<'_, UnsignedEvent> {
        Box::pin(async move {
            // Get the number of available CPU cores

            let num_threads = thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(1);

            // Single thread fallback
            if num_threads == 1 {
                return SingleThreadPow
                    .compute(unsigned_event, target_difficulty, custom_created_at)
                    .await;
            }

            let found: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
            let mut handles: Vec<JoinHandle<Option<UnsignedEvent>>> =
                Vec::with_capacity(num_threads);

            // Spawn threads
            for thread_id in 0..num_threads {
                let found: Arc<AtomicBool> = found.clone();
                let mut unsigned_event = unsigned_event.clone();

                let handle: JoinHandle<Option<UnsignedEvent>> = thread::spawn(move || {
                    let mut nonce: u128 = thread_id as u128;
                    unsigned_event.tags.push(Tag::pow(0, target_difficulty));
                    let pow_tag_index = unsigned_event.tags.len() - 1;

                    loop {
                        nonce += num_threads as u128;

                        // FIXME: "Division is the most expensive integer
                        // operation you can ask of your CPU"
                        // https://research.swtch.com/divmult
                        if (nonce / num_threads as u128) % 1024 == 0 {
                            // Check if another thread found the solution
                            if found.load(Ordering::Relaxed) {
                                break;
                            }
                            unsigned_event.created_at =
                                custom_created_at.unwrap_or_else(Timestamp::now);
                        }

                        unsigned_event.tags[pow_tag_index] = Tag::pow(nonce, target_difficulty);

                        let event_id = unsigned_event.compute_id();
                        if get_leading_zero_bits(event_id.as_bytes()) == target_difficulty {
                            found.store(true, Ordering::Relaxed);
                            unsigned_event.id = Some(event_id);
                            return Some(unsigned_event);
                        }
                    }

                    None
                });

                handles.push(handle);
            }

            // Wait for all threads to finish (non-blocking)
            loop {
                // TODO: yield to not block the async runtime
                // REF: https://github.com/rust-nostr/nostr/issues/921
                if found.load(Ordering::Relaxed) {
                    break;
                }
            }

            // Find result
            for handle in handles.into_iter() {
                // NOTE: this shouldn't block the current thread,
                // since above we've checked if the solution has been found
                // (so all threads should be terminated).
                if let Ok(Some(unsigned)) = handle.join() {
                    return unsigned;
                }
            }

            // Single thread fallback
            SingleThreadPow
                .compute(unsigned_event, target_difficulty, custom_created_at)
                .await
        })
    }
}

/// Returns the default single-threaded Proof-of-Work adapter.
#[cfg(not(feature = "pow-multi-thread"))]
#[inline]
pub(crate) fn default_adapter() -> SingleThreadPow {
    SingleThreadPow
}

/// Returns the default multi-threaded Proof-of-Work adapter.
#[cfg(feature = "pow-multi-thread")]
#[inline]
pub(crate) fn default_adapter() -> MultiThreadPow {
    MultiThreadPow
}

#[cfg(test)]
pub mod tests {
    use core::str::FromStr;
    #[cfg(feature = "std")]
    use std::sync::Arc;

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

    #[tokio::test]
    #[cfg(feature = "std")]
    async fn custom_adapter() {
        #[derive(Debug)]
        struct TestAdapter;

        impl PowAdapter for TestAdapter {
            fn compute(
                &self,
                mut unsigned_event: UnsignedEvent,
                target_difficulty: u8,
                _custom_created_at: Option<Timestamp>,
            ) -> BoxedFuture<'_, UnsignedEvent> {
                Box::pin(async move {
                    unsigned_event.tags.push(Tag::pow(3490, target_difficulty));
                    unsigned_event.ensure_id();

                    unsigned_event
                })
            }
        }

        let pow_adapter = Arc::new(TestAdapter);

        let unsigned = EventBuilder::text_note(
            "Why must I find leading zero bits? Is there no beauty in the ones?",
        )
        .pow_adapter(pow_adapter.clone())
        .pow(2)
        .build(PublicKey::from_slice(&[0; 32]).unwrap())
        .await;

        let Some(nonce_tag) = unsigned.tags.find(TagKind::Nonce) else {
            panic!("nonce tag should be exist")
        };

        assert_eq!(nonce_tag.as_slice()[1], "3490");
        assert_eq!(nonce_tag.as_slice()[2], "2");
    }

    #[tokio::test]
    #[cfg(feature = "std")]
    async fn single_adapter() {
        let unsigned = EventBuilder::text_note(
            "Proof of Work: The only workout my CPU gets since I stopped gaming",
        )
        .pow_adapter(Arc::new(SingleThreadPow))
        .pow(2)
        .build(PublicKey::from_slice(&[0; 32]).unwrap())
        .await;

        let Some(nonce_tag) = unsigned.tags.find(TagKind::Nonce) else {
            panic!("nonce tag should be exist")
        };

        assert!(unsigned.id.is_some());
        assert_eq!(nonce_tag.as_slice()[2], "2");
        assert_eq!(get_leading_zero_bits(unsigned.id.unwrap()), 2)
    }

    #[tokio::test]
    #[cfg(feature = "pow-multi-thread")]
    async fn threaded_adapter() {
        let unsigned = EventBuilder::text_note("Wait, you guys are getting paid to find nonces? I'm just doing it for the leading zeros")
            .pow_adapter(Arc::new(MultiThreadPow))
            .pow(2)
            .build(PublicKey::from_slice(&[0; 32]).unwrap())
            .await;

        let Some(nonce_tag) = unsigned.tags.find(TagKind::Nonce) else {
            panic!("nonce tag should be exist")
        };

        assert!(unsigned.id.is_some());
        assert_eq!(nonce_tag.as_slice()[2], "2");
        assert_eq!(get_leading_zero_bits(unsigned.id.unwrap()), 2)
    }
}
