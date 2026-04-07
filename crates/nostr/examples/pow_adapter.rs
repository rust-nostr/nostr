use core::num::NonZeroU8;
use std::time::Duration;

use nostr::prelude::*;

#[derive(Debug)]
enum MyPowError {
    Timeout,
}

impl core::error::Error for MyPowError {}

impl core::fmt::Display for MyPowError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Timeout => f.write_str("pow timeout"),
        }
    }
}

#[derive(Debug)]
struct MyAdapter;

impl PowAdapter for MyAdapter {
    type Error = MyPowError;

    fn compute(
        &self,
        _unsigned_event: UnsignedEvent,
        _difficulty: NonZeroU8,
    ) -> Result<UnsignedEvent, Self::Error> {
        std::thread::sleep(Duration::from_secs(5));

        // Simulate always error
        Err(MyPowError::Timeout)
    }
}

fn main() {
    let keys = Keys::generate();
    let _event = EventBuilder::text_note("test")
        .pow(NonZeroU8::new(20).unwrap(), MyAdapter)
        .finalize(&keys)
        .unwrap();
}
