use core::convert::Infallible;
use core::num::NonZeroU8;
use core::sync::atomic::{AtomicBool, Ordering};

#[cfg(feature = "std")]
use super::AsyncPowAdapter;
use super::PowAdapter;
#[cfg(feature = "std")]
use super::blocking_wrapper::BlockingPowFuture;
#[cfg(feature = "std")]
use crate::util::BoxedFuture;
use crate::{Tag, UnsignedEvent};

/// A single-threaded PoW miner implementation
#[derive(Debug)]
pub struct SingleThreadPow;

impl PowAdapter for SingleThreadPow {
    type Error = Infallible;

    #[inline]
    fn compute(
        &self,
        unsigned: UnsignedEvent,
        target_difficulty: NonZeroU8,
    ) -> Result<UnsignedEvent, Self::Error> {
        Ok(mine(unsigned, target_difficulty.get(), None).unwrap())
    }
}

#[cfg(feature = "std")]
impl AsyncPowAdapter for SingleThreadPow {
    type Error = Infallible;

    fn compute(
        &self,
        unsigned: UnsignedEvent,
        target_difficulty: NonZeroU8,
    ) -> BoxedFuture<'_, Result<UnsignedEvent, Self::Error>> {
        let diff: u8 = target_difficulty.get();

        Box::pin(async move {
            // If the future is dropped (cancelled), BlockingPowFuture's Drop sets the cancel
            // flag, mine() returns None, and we never reach this line.
            Ok(
                BlockingPowFuture::new(Box::new(move |cancel| mine(unsigned, diff, Some(&cancel))))
                    .await
                    .unwrap(),
            )
        })
    }
}

pub(super) fn mine(
    mut unsigned: UnsignedEvent,
    difficulty: u8,
    cancel: Option<&AtomicBool>,
) -> Option<UnsignedEvent> {
    let mut nonce: u128 = 0;
    unsigned.tags.push(Tag::pow(0, difficulty));
    let pow_tag_index = unsigned.tags.len() - 1;

    loop {
        if let Some(cancel) = cancel {
            if nonce % 1024 == 0 && cancel.load(Ordering::SeqCst) {
                return None;
            }
        }

        nonce += 1;
        unsigned.tags[pow_tag_index] = Tag::pow(nonce, difficulty);
        let event_id = unsigned.compute_id();

        if event_id.check_pow(difficulty) {
            unsigned.id = Some(event_id);
            return Some(unsigned);
        }
    }
}

#[cfg(test)]
#[cfg(feature = "std")]
pub mod tests {
    use std::sync::Arc;
    use std::sync::atomic::AtomicBool;
    use std::thread;
    use std::time::Duration;

    use super::*;
    use crate::prelude::*;

    #[test]
    fn single_adapter() {
        let unsigned = EventBuilder::text_note(
            "Proof of Work: The only workout my CPU gets since I stopped gaming",
        )
        .build(PublicKey::from_slice(&[0; 32]).unwrap());

        // Mine the event
        let unsigned = unsigned
            .mine(&SingleThreadPow, NonZeroU8::new(2).unwrap())
            .unwrap();

        let Some(nonce_tag) = unsigned.tags.find(TagKind::Nonce) else {
            panic!("nonce tag should be exist")
        };

        assert!(unsigned.id.is_some());
        assert_eq!(nonce_tag.as_slice()[2], "2");
        assert!(get_leading_zero_bits(unsigned.id.unwrap()) >= 2)
    }

    #[test]
    fn single_thread_mining_can_be_cancelled() {
        let cancel: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
        let unsigned = EventBuilder::text_note("single thread cancellation test")
            .build(PublicKey::from_slice(&[0; 32]).unwrap());

        let worker_cancel: Arc<AtomicBool> = cancel.clone();
        let handle = thread::spawn(move || mine(unsigned, u8::MAX, Some(worker_cancel.as_ref())));

        thread::sleep(Duration::from_millis(10));
        cancel.store(true, Ordering::SeqCst);

        let result = handle.join().expect("single thread miner should not panic");
        assert!(
            result.is_none(),
            "single thread miner should stop on cancel"
        );
    }
}
