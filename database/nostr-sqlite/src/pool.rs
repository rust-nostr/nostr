use std::path::Path;
#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;
use std::sync::Arc;

#[cfg(not(target_arch = "wasm32"))]
use async_utility::task;
use rusqlite::{Connection, OpenFlags};
use tokio::sync::Mutex;

use crate::error::Error;
use crate::store::NostrSqliteOptions;

#[derive(Debug, Clone)]
pub(crate) struct Pool {
    conn: Arc<Mutex<Connection>>,
    options: Arc<NostrSqliteOptions>,
}

impl Pool {
    fn new(conn: Connection, options: NostrSqliteOptions) -> Self {
        Self {
            conn: Arc::new(Mutex::new(conn)),
            options: Arc::new(options),
        }
    }

    pub(crate) fn open_in_memory(options: NostrSqliteOptions) -> Result<Self, Error> {
        let conn: Connection = Connection::open_in_memory()?;
        Ok(Self::new(conn, options))
    }

    #[inline]
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) async fn open_with_path(
        path: PathBuf,
        options: NostrSqliteOptions,
    ) -> Result<Self, Error> {
        let conn: Connection = task::spawn_blocking(move || Connection::open(path)).await??;
        Ok(Self::new(conn, options))
    }

    pub(crate) async fn open_with_vfs<P>(
        path: P,
        vfs: &str,
        options: NostrSqliteOptions,
    ) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        let flags = OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE;
        let conn: Connection = Connection::open_with_flags_and_vfs(path, flags, vfs)?;
        Ok(Self::new(conn, options))
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub async fn interact<F, R>(&self, f: F) -> Result<R, Error>
    where
        F: FnOnce(&mut Connection) -> Result<R, Error> + Send + 'static,
        R: Send + 'static,
    {
        let arc: Arc<Mutex<Connection>> = self.conn.clone();
        let mut conn = arc.lock_owned().await;
        task::spawn_blocking(move || f(&mut conn)).await?
    }

    #[cfg(target_arch = "wasm32")]
    pub async fn interact<F, R>(&self, f: F) -> Result<R, Error>
    where
        F: FnOnce(&mut Connection) -> Result<R, Error> + 'static,
        R: 'static,
    {
        let mut conn = self.conn.lock().await;
        f(&mut conn)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub async fn interact_options<F, R>(&self, f: F) -> Result<R, Error>
    where
        F: FnOnce(&mut Connection, &NostrSqliteOptions) -> Result<R, Error> + Send + 'static,
        R: Send + 'static,
    {
        let arc: Arc<Mutex<Connection>> = self.conn.clone();
        let mut conn = arc.lock_owned().await;
        let options: Arc<NostrSqliteOptions> = self.options.clone();
        task::spawn_blocking(move || f(&mut conn, &options)).await?
    }

    #[cfg(target_arch = "wasm32")]
    pub async fn interact_options<F, R>(&self, f: F) -> Result<R, Error>
    where
        F: FnOnce(&mut Connection, &NostrSqliteOptions) -> Result<R, Error> + 'static,
        R: 'static,
    {
        let mut conn = self.conn.lock().await;
        f(&mut conn, &self.options)
    }
}
