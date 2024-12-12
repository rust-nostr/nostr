// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::sync::Arc;

use async_utility::task;
use rusqlite::Connection;
use tokio::sync::Mutex;

use super::error::Error;

#[derive(Debug, Clone)]
pub(crate) struct Pool {
    conn: Arc<Mutex<Connection>>,
}

impl Pool {
    #[inline]
    pub fn new(conn: Connection) -> Self {
        Self {
            conn: Arc::new(Mutex::new(conn)),
        }
    }

    pub async fn interact<F, R>(&self, f: F) -> Result<R, Error>
    where
        F: FnOnce(&mut Connection) -> R + Send + 'static,
        R: Send + 'static,
    {
        let arc: Arc<Mutex<Connection>> = self.conn.clone();
        let mut conn = arc.lock_owned().await;
        Ok(task::spawn_blocking(move || f(&mut conn)).await?)
    }
}
