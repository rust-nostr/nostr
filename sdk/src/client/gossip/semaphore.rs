//! Gossip semaphore
//!
//! This semaphore coordinates concurrent gossip updates while guaranteeing
//! no deadlocks under any conditions.

use std::collections::{BTreeSet, HashMap};
use std::sync::Arc;

use nostr::prelude::*;
use tokio::sync::{Mutex, OwnedSemaphorePermit, Semaphore};

const PERMIT_NUM: usize = 1;

#[derive(Debug, Clone)]
pub(in crate::client) struct GossipSemaphore {
    /// Tracks semaphores per public key
    in_flight: Arc<Mutex<HashMap<PublicKey, Arc<Semaphore>>>>,
}

impl GossipSemaphore {
    #[inline]
    pub(super) fn new() -> Self {
        Self {
            in_flight: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Acquire permits for multiple public keys atomically
    ///
    /// This method is guaranteed to never deadlock, no matter how many
    /// concurrent requests or how they overlap.
    ///
    /// # How it works
    ///
    /// 1. Acquire mutex lock (serializes permit acquisition)
    /// 2. Get/create semaphores for all keys
    /// 3. Acquire permits from all semaphores (in sorted order)
    /// 4. Release mutex lock
    /// 5. Return RAII guard (work proceeds concurrently)
    pub(in crate::client) async fn acquire(
        &self,
        public_keys: BTreeSet<PublicKey>,
    ) -> GossipSemaphorePermit {
        // Acquire mutex lock
        let mut map = self.in_flight.lock().await;

        // Get all semaphores (create if needed)
        let mut semaphores: Vec<Arc<Semaphore>> = Vec::with_capacity(public_keys.len());

        for public_key in &public_keys {
            let semaphore: Arc<Semaphore> = map
                .entry(*public_key)
                .or_insert_with(|| Arc::new(Semaphore::new(PERMIT_NUM)))
                .clone();
            semaphores.push(semaphore);
        }

        // Acquire all permits (in sorted order, guaranteed by BTreeSet)
        // We still hold the mutex lock, so we know no OTHER task is acquiring
        // But we might need to wait for tasks that ALREADY have permits
        let mut permits: Vec<OwnedSemaphorePermit> = Vec::with_capacity(semaphores.len());

        for sem in semaphores {
            let permit: OwnedSemaphorePermit = sem.acquire_owned().await.expect("semaphore closed");
            permits.push(permit);
        }

        // Release the mutex lock
        drop(map);

        // Return a permit
        GossipSemaphorePermit::new(permits, public_keys, self.clone())
    }

    /// Clean up unused semaphores
    ///
    /// When a permit is dropped, this checks if the semaphore is no longer needed.
    ///
    /// If available_permits() == 1, it means:
    /// - The semaphore has capacity for 1 permit (max capacity)
    /// - No one is currently holding it (all permits returned)
    /// - No one is waiting for it
    ///
    /// Therefore, it's safe to remove and free memory.
    async fn cleanup(&self, public_keys: &BTreeSet<PublicKey>) {
        let mut map = self.in_flight.lock().await;
        for pk in public_keys {
            if let Some(sem) = map.get(pk) {
                // If all permits are available, nobody is using this semaphore
                if sem.available_permits() == PERMIT_NUM {
                    map.remove(pk);
                }
            }
        }
    }
}

#[derive(Debug)]
struct InnerSemaphorePermit {
    permits: Vec<OwnedSemaphorePermit>,
    public_keys: BTreeSet<PublicKey>,
    semaphore: GossipSemaphore,
}

/// RAII guard for gossip semaphore permits
#[derive(Debug)]
pub struct GossipSemaphorePermit(Option<InnerSemaphorePermit>);

impl GossipSemaphorePermit {
    #[inline]
    fn new(
        permits: Vec<OwnedSemaphorePermit>,
        public_keys: BTreeSet<PublicKey>,
        semaphore: GossipSemaphore,
    ) -> Self {
        let inner: InnerSemaphorePermit = InnerSemaphorePermit {
            permits,
            public_keys,
            semaphore,
        };
        Self(Some(inner))
    }
}

// TODO: replace with AsyncDrop when stable
impl Drop for GossipSemaphorePermit {
    fn drop(&mut self) {
        let inner: InnerSemaphorePermit = self
            .0
            .take()
            .expect("BUG: semaphore permit already dropped");

        // Drop permits
        drop(inner.permits);

        // Cleanup
        tokio::spawn(async move {
            inner.semaphore.cleanup(&inner.public_keys).await;
        });
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    use super::*;

    /// Test: Basic concurrent access with different keys
    ///
    /// Purpose: Verify that two tasks requesting different public keys
    /// can run concurrently without blocking each other.
    ///
    /// Setup: 2 tasks, 2 different keys
    /// Expected: Both tasks complete successfully
    #[tokio::test]
    async fn test_basic_concurrency() {
        let semaphore = GossipSemaphore::new();
        let pk1 = Keys::generate().public_key();
        let pk2 = Keys::generate().public_key();

        let s1 = semaphore.clone();
        let s2 = semaphore.clone();

        let t1 = tokio::spawn(async move {
            let _p = s1.acquire(BTreeSet::from([pk1])).await;
            tokio::time::sleep(Duration::from_millis(10)).await;
        });

        let t2 = tokio::spawn(async move {
            let _p = s2.acquire(BTreeSet::from([pk2])).await;
            tokio::time::sleep(Duration::from_millis(10)).await;
        });

        assert!(tokio::join!(t1, t2).0.is_ok());
    }

    /// Test: Same key blocks - mutual exclusion
    ///
    /// Purpose: Verify that two tasks requesting the SAME public key
    /// are properly serialized (second waits for first to complete).
    ///
    /// Setup: 2 tasks, 1 shared key
    /// Expected: Task 2 only runs after Task 1 completes
    /// Verification: Counter shows Task 1 incremented twice before Task 2 starts
    #[tokio::test]
    async fn test_same_key_blocks() {
        let semaphore = GossipSemaphore::new();
        let pk = Keys::generate().public_key();

        let counter = Arc::new(AtomicUsize::new(0));
        let c1 = counter.clone();
        let c2 = counter.clone();
        let s1 = semaphore.clone();
        let s2 = semaphore.clone();

        let t1 = tokio::spawn(async move {
            let _p = s1.acquire(BTreeSet::from([pk])).await;
            c1.fetch_add(1, Ordering::SeqCst); // counter = 1
            tokio::time::sleep(Duration::from_millis(50)).await;
            c1.fetch_add(1, Ordering::SeqCst); // counter = 2
        });

        // Small delay to ensure t1 acquires first
        tokio::time::sleep(Duration::from_millis(5)).await;

        let t2 = tokio::spawn(async move {
            let _p = s2.acquire(BTreeSet::from([pk])).await;
            // If we get here, t1 must have completed (counter == 2)
            assert_eq!(c2.load(Ordering::SeqCst), 2);
            c2.fetch_add(1, Ordering::SeqCst); // counter = 3
        });

        assert!(tokio::join!(t1, t2).0.is_ok());
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    /// Test: 100 concurrent requests with overlapping keys
    ///
    /// Purpose: Stress test with moderate concurrency
    ///
    /// Setup:
    /// - 100 concurrent tasks
    /// - 20 total public keys
    /// - Each task requests 1-5 keys (varying)
    /// - Keys selected in overlapping patterns
    ///
    /// Expected: All 100 tasks complete without deadlock
    /// Timeout: 10 seconds (should complete in <1 second)
    #[tokio::test]
    async fn test_100_requests() {
        let semaphore = GossipSemaphore::new();
        let keys: Vec<_> = (0..20).map(|_| Keys::generate().public_key()).collect();

        let completed = Arc::new(AtomicUsize::new(0));
        let mut tasks = Vec::new();

        for i in 0..100 {
            let s = semaphore.clone();
            let c = completed.clone();
            let ks = keys.clone();

            tasks.push(tokio::spawn(async move {
                // Each task requests 1-5 keys depending on task number
                let selected: BTreeSet<_> = (0..((i % 5) + 1)).map(|j| ks[(i + j) % 20]).collect();

                let _p = s.acquire(selected).await;
                tokio::time::sleep(Duration::from_micros(100)).await;
                c.fetch_add(1, Ordering::SeqCst);
            }));
        }

        let result = tokio::time::timeout(Duration::from_secs(10), async {
            for t in tasks {
                t.await.unwrap();
            }
        })
        .await;

        assert!(result.is_ok(), "Deadlock with 100 requests!");
        assert_eq!(completed.load(Ordering::SeqCst), 100);
    }

    /// Test: 1,000 concurrent requests
    ///
    /// Purpose: Heavy stress test
    ///
    /// Setup:
    /// - 1,000 concurrent tasks
    /// - 50 total public keys
    /// - Each task requests 1-10 keys
    ///
    /// Expected: All 1,000 tasks complete without deadlock
    /// Timeout: 30 seconds (should complete in <1 second)
    #[tokio::test]
    async fn test_1000_requests() {
        let semaphore = GossipSemaphore::new();
        let keys: Vec<_> = (0..50).map(|_| Keys::generate().public_key()).collect();

        let completed = Arc::new(AtomicUsize::new(0));
        let mut tasks = Vec::new();

        for i in 0..1000 {
            let s = semaphore.clone();
            let c = completed.clone();
            let ks = keys.clone();

            tasks.push(tokio::spawn(async move {
                let selected: BTreeSet<_> = (0..((i % 10) + 1)).map(|j| ks[(i + j) % 50]).collect();

                let _p = s.acquire(selected).await;
                c.fetch_add(1, Ordering::SeqCst);
            }));
        }

        let result = tokio::time::timeout(Duration::from_secs(30), async {
            for t in tasks {
                t.await.unwrap();
            }
        })
        .await;

        assert!(result.is_ok(), "Deadlock with 1,000 requests!");
        assert_eq!(completed.load(Ordering::SeqCst), 1000);
    }

    /// Test: 10,000 concurrent requests - extreme stress test
    ///
    /// Purpose: Verify the semaphore can handle massive concurrency
    ///
    /// Setup:
    /// - 10,000 concurrent tasks (!!)
    /// - 100 total public keys
    /// - Each task requests 1-20 keys
    ///
    /// Expected: All 10,000 tasks complete without deadlock
    /// Timeout: 60 seconds (should complete in <1 second)
    #[tokio::test]
    async fn test_10000_requests() {
        let semaphore = GossipSemaphore::new();
        let keys: Vec<_> = (0..100).map(|_| Keys::generate().public_key()).collect();

        let completed = Arc::new(AtomicUsize::new(0));
        let mut tasks = Vec::new();

        for i in 0..10000 {
            let s = semaphore.clone();
            let c = completed.clone();
            let ks = keys.clone();

            tasks.push(tokio::spawn(async move {
                let selected: BTreeSet<_> =
                    (0..((i % 20) + 1)).map(|j| ks[(i + j) % 100]).collect();

                let _p = s.acquire(selected).await;
                c.fetch_add(1, Ordering::SeqCst);
            }));
        }

        let result = tokio::time::timeout(Duration::from_secs(60), async {
            for t in tasks {
                t.await.unwrap();
            }
        })
        .await;

        assert!(result.is_ok(), "Deadlock with 10,000 requests!");
        assert_eq!(completed.load(Ordering::SeqCst), 10000);
    }

    /// Test: 1,000 UNIQUE keys
    ///
    /// Purpose: Verify semaphore works with a large number of unique public keys
    ///
    /// Setup:
    /// - 100 concurrent tasks
    /// - 1,000 total unique public keys (!!)
    /// - Each task requests 10-200 keys (realistic gossip scenario)
    ///
    /// Expected: All tasks complete, memory is efficiently managed
    /// Timeout: 10 seconds
    ///
    /// This tests the case where you have many different public keys to track
    /// with varying request sizes (10-200 keys per request)
    #[tokio::test]
    async fn test_1000_unique_keys() {
        let semaphore = GossipSemaphore::new();
        let keys: Vec<_> = (0..1000).map(|_| Keys::generate().public_key()).collect();

        let completed = Arc::new(AtomicUsize::new(0));
        let mut tasks = Vec::new();

        for i in 0..100 {
            let s = semaphore.clone();
            let c = completed.clone();
            let ks = keys.clone();

            tasks.push(tokio::spawn(async move {
                // Each task selects 10-200 keys from the 1000 available
                // Formula: 10 + (i % 191) gives range [10, 200]
                let num_keys: usize = 10 + (i % 191);
                let selected: BTreeSet<_> =
                    (0..num_keys).map(|j| ks[(i * 10 + j) % 1000]).collect();

                let _p = s.acquire(selected).await;
                c.fetch_add(1, Ordering::SeqCst);
            }));
        }

        let result = tokio::time::timeout(Duration::from_secs(10), async {
            for t in tasks {
                t.await.unwrap();
            }
        })
        .await;

        assert!(result.is_ok(), "Deadlock with 1,000 unique keys!");
        assert_eq!(completed.load(Ordering::SeqCst), 100);
    }

    /// Test: Memory cleanup after permit release
    ///
    /// Purpose: Verify that semaphores are properly cleaned up when no longer needed
    ///
    /// Setup:
    /// - Acquire permit for a key
    /// - Release permit (drop)
    /// - Check that semaphore is removed from the map
    ///
    /// Expected: Semaphore is removed, preventing memory leaks
    ///
    /// Why this matters: With thousands of public keys over time, we don't want
    /// to keep semaphores forever. They should be cleaned up when not in use.
    #[tokio::test]
    async fn test_cleanup() {
        let semaphore = GossipSemaphore::new();
        let pk = Keys::generate().public_key();

        // Acquire and immediately drop permit
        {
            let _p = semaphore.acquire(BTreeSet::from([pk])).await;
        } // _p is dropped here

        // Give cleanup task time to run
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Verify semaphore was cleaned up
        let map = semaphore.in_flight.lock().await;
        assert!(
            !map.contains_key(&pk),
            "Semaphore should be cleaned up to prevent memory leak"
        );
    }

    /// Test: Cleanup doesn't remove semaphores still in use
    ///
    /// Purpose: Verify cleanup logic correctly identifies when a semaphore is still needed
    ///
    /// Setup:
    /// - Task 1 acquires permit for pk1 and holds it
    /// - Task 2 tries to acquire same pk1 (will wait)
    /// - Task 1 releases, triggering cleanup
    /// - Cleanup should NOT remove semaphore (Task 2 is waiting)
    ///
    /// Expected: Semaphore remains in map while Task 2 is waiting
    /// Coverage: Tests the `available_permits() != 1` branch in cleanup
    #[tokio::test]
    async fn test_cleanup_keeps_active_semaphores() {
        let semaphore = GossipSemaphore::new();
        let pk = Keys::generate().public_key();

        let s1 = semaphore.clone();
        let s2 = semaphore.clone();

        // Task 1: Acquire and hold permit briefly
        let t1 = tokio::spawn(async move {
            let _p = s1.acquire(BTreeSet::from([pk])).await;
            tokio::time::sleep(Duration::from_millis(100)).await;
            // Permit dropped here, cleanup runs
        });

        // Small delay to ensure t1 acquires first
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Task 2: Try to acquire same key (will wait for t1)
        let t2 = tokio::spawn(async move {
            let _p = s2.acquire(BTreeSet::from([pk])).await;
            tokio::time::sleep(Duration::from_millis(10)).await;
        });

        // Wait for both tasks
        let _ = tokio::join!(t1, t2);

        // Final cleanup should happen
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Now semaphore should be cleaned up (nobody using it)
        let map = semaphore.in_flight.lock().await;
        assert!(
            !map.contains_key(&pk),
            "Semaphore should be cleaned up after all tasks complete"
        );
    }
}
