//! Nostr SQLite database builder

use std::path::{Path, PathBuf};

use crate::error::Error;
use crate::store::NostrSqlite;

#[derive(Default)]
pub(crate) enum DatabaseConnType {
    #[default]
    InMemory,
    #[cfg(not(target_arch = "wasm32"))]
    File(PathBuf),
    WithVFS {
        path: PathBuf,
        vfs: String,
    },
}

/// Builder for [`NostrSqlite`] database.
#[derive(Default)]
pub struct NostrSqliteBuilder {
    /// Database connection type. In-memory by default
    pub(crate) db_type: DatabaseConnType,
}

impl NostrSqliteBuilder {
    /// Use an in-memory database.
    ///
    /// This overrides any previous connection setting.
    #[inline]
    pub fn in_memory(mut self) -> Self {
        self.db_type = DatabaseConnType::InMemory;
        self
    }

    /// Connect to a SQLite database file.
    ///
    /// This overrides any previous connection setting.
    #[inline]
    #[cfg(not(target_arch = "wasm32"))]
    pub fn in_file<P>(mut self, path: P) -> Self
    where
        P: AsRef<Path>,
    {
        self.db_type = DatabaseConnType::File(path.as_ref().to_path_buf());
        self
    }

    /// Connect to a SQLite database file using a specific VFS.
    ///
    /// This overrides any previous connection setting.
    #[inline]
    pub fn persistent_with_vfs<P, S>(mut self, path: P, vfs: S) -> Self
    where
        P: AsRef<Path>,
        S: Into<String>,
    {
        self.db_type = DatabaseConnType::WithVFS {
            path: path.as_ref().to_path_buf(),
            vfs: vfs.into(),
        };
        self
    }

    /// Build [`NostrSqlite`] database
    #[inline]
    pub async fn build(self) -> Result<NostrSqlite, Error> {
        NostrSqlite::from_builder(self).await
    }
}
