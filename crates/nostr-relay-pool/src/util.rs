// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::sync::atomic::{AtomicUsize, Ordering};

pub trait SaturatingUsize {
    fn saturating_increment(&self, order: Ordering) -> usize;

    fn saturating_decrement(&self, order: Ordering) -> usize;
}

impl SaturatingUsize for AtomicUsize {
    /// Atomically increments the AtomicUsize by 1, saturating at `usize::MAX`.
    ///
    /// Return the new value or `usize::MAX`.
    fn saturating_increment(&self, order: Ordering) -> usize {
        loop {
            let current: usize = self.load(order);

            if current == usize::MAX {
                // Already at maximum, cannot increment further
                return current;
            }

            let new: usize = current + 1;
            match self.compare_exchange(current, new, order, order) {
                Ok(_) => return new,
                Err(_) => continue, // Retry if the value changed concurrently
            }
        }
    }

    /// Atomically decrements the AtomicUsize by 1, saturating at `0`.
    ///
    /// Return the new value or `0`.
    fn saturating_decrement(&self, order: Ordering) -> usize {
        loop {
            let current: usize = self.load(order);

            if current == 0 {
                // Already at minimum, cannot decrement further
                return current;
            }

            let new: usize = current - 1;
            match self.compare_exchange(current, new, order, order) {
                Ok(_) => return new,
                Err(_) => continue, // Retry if the value changed concurrently
            }
        }
    }
}
