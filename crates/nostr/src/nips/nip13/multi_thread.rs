use std::convert::Infallible;
use std::num::NonZeroU8;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use super::{
    AsyncPowAdapter, BoxedFuture, BoxedFuture, PowAdapter, PowAdapter, SingleThreadPow,
    get_leading_zero_bits, get_leading_zero_bits, get_prefixes_for_difficulty, single_thread,
};
use crate::{Tag, UnsignedEvent};

/// A multi-threaded Proof-of-Work miner.
///
/// Fallback to [`SingleThreadPow`] if:
/// - the number of threads is `1`;
/// - thread spawning or coordination fails;
/// - no valid solution is found by any thread (rare edge case)
#[derive(Debug)]
pub struct MultiThreadPow;

impl PowAdapter for MultiThreadPow {
    type Error = Infallible;

    fn compute(
        &self,
        unsigned_event: UnsignedEvent,
        target_difficulty: NonZeroU8,
    ) -> Result<UnsignedEvent, Self::Error> {
        let cancel = AtomicBool::new(false);
        Ok(mine(unsigned_event, target_difficulty.get(), cancel).unwrap())
    }
}

impl AsyncPowAdapter for MultiThreadPow {
    type Error = Infallible;

    fn compute_async(
        &self,
        unsigned_event: UnsignedEvent,
        target_difficulty: NonZeroU8,
    ) -> BoxedFuture<'_, Result<UnsignedEvent, Self::Error>> {
        let diff: u8 = target_difficulty.get();

        Box::pin(async move {
            Ok(
                BlockingPowFuture::new(Box::new(move |cancel| mine(unsigned_event, diff, cancel)))
                    .await
                    .unwrap(),
            )
        })
    }
}

fn mine(
    unsigned_event: UnsignedEvent,
    difficulty: u8,
    cancel: Arc<AtomicBool>,
) -> Option<UnsignedEvent> {
    let num_threads = thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);

    if num_threads == 1 {
        return single_thread::mine(unsigned_event, difficulty, &cancel);
    }

    let found: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    let mut handles: Vec<JoinHandle<Option<UnsignedEvent>>> = Vec::with_capacity(num_threads);

    for thread_id in 0..num_threads {
        let found: Arc<AtomicBool> = found.clone();
        let cancel: Arc<AtomicBool> = cancel.clone();

        let mut event: UnsignedEvent = unsigned_event.clone();

        let handle: JoinHandle<Option<UnsignedEvent>> = thread::spawn(move || {
            let mut nonce: u128 = thread_id as u128;
            event.tags.push(Tag::pow(0, difficulty));
            let pow_tag_index = event.tags.len() - 1;

            loop {
                nonce += num_threads as u128;

                // FIXME: "Division is the most expensive integer
                // operation you can ask of your CPU"
                // https://research.swtch.com/divmult
                if (nonce / num_threads as u128) % 1024 == 0 {
                    if found.load(Ordering::SeqCst) || cancel.load(Ordering::SeqCst) {
                        return None;
                    }
                }

                event.tags[pow_tag_index] = Tag::pow(nonce, difficulty);
                let event_id = event.compute_id();

                if id.check_pow(difficulty) {
                    found.store(true, Ordering::SeqCst);
                    event.id = Some(event_id);
                    return Some(event);
                }
            }
        });

        handles.push(handle);
    }

    // Wait for found or cancel
    loop {
        if found.load(Ordering::SeqCst) || cancel.load(Ordering::SeqCst) {
            break;
        }

        thread::sleep(Duration::from_millis(100));
    }

    if !found.load(Ordering::SeqCst) {
        return None;
    }

    for handle in handles {
        if let Ok(Some(event)) = handle.join() {
            return Some(event);
        }
    }

    // Fallback: shouldn't be reached since found=true guarantees one thread returned Some
    single_thread::mine(unsigned_event, difficulty, &cancel)
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[tokio::test]
    async fn threaded_adapter() {
        let unsigned = EventBuilder::text_note("Wait, you guys are getting paid to find nonces? I'm just doing it for the leading zeros")
            .pow(NonZeroU8::new(2).unwrap(), MultiThreadPow)
            .build(PublicKey::from_slice(&[0; 32]).unwrap())
            .unwrap();

        let Some(nonce_tag) = unsigned.tags.find(TagKind::Nonce) else {
            panic!("nonce tag should be exist")
        };

        assert!(unsigned.id.is_some());
        assert_eq!(nonce_tag.as_slice()[2], "2");
        assert!(get_leading_zero_bits(unsigned.id.unwrap()) >= 2)
    }
}
