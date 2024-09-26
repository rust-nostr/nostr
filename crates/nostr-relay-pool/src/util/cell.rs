// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::fmt::Debug;
use std::future::Future;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::Mutex;

#[derive(Debug, Clone, Copy)]
struct Item<T> {
    value: T,
    expiry: Instant,
}

#[derive(Debug, Clone)]
pub struct TimedOnceCell<T>
where
    T: Debug + Clone + Copy,
{
    value: Arc<Mutex<Option<Item<T>>>>,
    expire_after: Duration,
}

impl<T> TimedOnceCell<T>
where
    T: Debug + Clone + Copy,
{
    /// Create a cell that expire after [`Duration`]
    #[inline]
    pub fn new(expire_after: Duration) -> Self {
        Self {
            value: Arc::new(Mutex::new(None)),
            expire_after,
        }
    }

    pub async fn get_or_try_init<E, F, Fut>(&self, f: F) -> Result<T, E>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T, E>>,
    {
        let mut item = self.value.lock().await;

        // Take value
        match &*item {
            // If `expiry == None` OR `now >= expiry` -> call function and set value + expiry
            Some(item) if item.expiry > Instant::now() => Ok(item.value),
            _ => {
                let value: T = f().await?;
                *item = Some(Item {
                    value,
                    expiry: Instant::now() + self.expire_after,
                });
                Ok(value)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use async_utility::thread;

    use super::*;

    async fn some_future() -> Result<bool, ()> {
        thread::sleep(Duration::from_secs(1)).await; // To mimic some delay in data available from Future
        Ok(true)
    }

    async fn some_future_refresh() -> Result<bool, ()> {
        thread::sleep(Duration::from_secs(1)).await; // To mimic some delay in data available from Future
        Ok(false)
    }

    #[tokio::test]
    async fn test_value_refresh_after_expiry() {
        let cell: TimedOnceCell<bool> = TimedOnceCell::new(Duration::from_secs(3));
        let val = cell.get_or_try_init(some_future).await.unwrap();
        assert_eq!(val, true, "Initial value not correctly set!");

        thread::sleep(Duration::from_secs(4)).await; // Add sleep to let the expiry time pass

        // At this point the cell value should be expired and new value through future should be fetched
        let next_val = cell.get_or_try_init(some_future_refresh).await.unwrap();
        assert_eq!(next_val, false, "Value not refreshed after expiry!");
    }

    #[tokio::test]
    async fn test_value_persists_within_expiry() {
        let cell: TimedOnceCell<bool> = TimedOnceCell::new(Duration::from_secs(5));
        let val = cell.get_or_try_init(some_future).await.unwrap();
        assert_eq!(val, true, "Initial value not correctly set!");

        thread::sleep(Duration::from_secs(2)).await; // Add sleep within expiry time limit

        // At this point the expiry time should not be past and the initial value should be fetched
        let same_val = cell.get_or_try_init(some_future_refresh).await.unwrap();
        assert_eq!(
            same_val, true,
            "Value not persisted within expiry time limit!"
        );
    }
}
