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
pub struct NostrSqliteBuilder {
    /// Database connection type. In-memory by default
    pub(crate) db_type: DatabaseConnType,
    /// Whether to process request to vanish (NIP-62) events
    pub(crate) process_nip62: bool,
    /// Whether to process event deletion request (NIP-09) events
    pub(crate) process_nip09: bool,
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

    /// Whether to process request to vanish (NIP-62) events
    ///
    /// Defaults to `true`
    #[inline]
    pub fn process_nip62(mut self, process_nip62: bool) -> Self {
        self.process_nip62 = process_nip62;
        self
    }

    /// Whether to process event deletion request (NIP-09) events
    ///
    /// Defaults to `true`
    #[inline]
    pub fn process_nip09(mut self, process_nip09: bool) -> Self {
        self.process_nip09 = process_nip09;
        self
    }

    /// Build [`NostrSqlite`] database
    #[inline]
    pub async fn build(self) -> Result<NostrSqlite, Error> {
        NostrSqlite::from_builder(self).await
    }
}

impl Default for NostrSqliteBuilder {
    /// Creates a new builder with default settings.
    ///
    /// This builder uses an in-memory database connection and enables
    /// processing for NIP-62 and NIP-09 features by default.
    fn default() -> Self {
        Self {
            db_type: Default::default(),
            process_nip62: true,
            process_nip09: true,
        }
    }
}
