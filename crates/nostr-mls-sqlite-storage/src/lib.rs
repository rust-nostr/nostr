//! SQLite-based storage implementation for Nostr MLS.
//!
//! This module provides a SQLite-based storage implementation for the Nostr MLS (Messaging Layer Security)
//! crate. It implements the [`NostrMlsStorageProvider`] trait, allowing it to be used within the Nostr MLS context.
//!
//! SQLite-based storage is persistent and will be saved to a file. It's useful for production applications
//! where data persistence is required.

use std::path::Path;
use std::sync::{Arc, Mutex};

use nostr_mls_storage::{Backend, NostrMlsStorageProvider};
use openmls_sqlite_storage::Codec;
use rusqlite::Connection;
use serde::Serialize;

mod db;
pub mod error;
mod groups;
mod messages;
mod migrations;
mod welcomes;

#[derive(Default)]
pub struct JsonCodec;

impl Codec for JsonCodec {
    type Error = serde_json::Error;

    fn to_vec<T: Serialize>(value: &T) -> Result<Vec<u8>, Self::Error> {
        serde_json::to_vec(value)
    }

    fn from_slice<T: serde::de::DeserializeOwned>(slice: &[u8]) -> Result<T, Self::Error> {
        serde_json::from_slice(slice)
    }
}

// Define a type alias for the specific SqliteStorageProvider we're using
type MlsStorage = openmls_sqlite_storage::SqliteStorageProvider<JsonCodec, Connection>;

/// A SQLite-based storage implementation for Nostr MLS.
///
/// This struct implements the NostrMlsStorageProvider trait for SQLite databases.
/// It directly interfaces with a SQLite database for storing MLS data.
pub struct NostrMlsSqliteStorage {
    /// The OpenMLS storage implementation
    openmls_storage: MlsStorage,
    /// The SQLite connection
    db_connection: Arc<Mutex<Connection>>,
}

impl NostrMlsSqliteStorage {
    /// Creates a new [`NostrMlsSqliteStorage`] with the provided file path.
    ///
    /// # Arguments
    ///
    /// * `file_path` - Path to the SQLite database file.
    ///
    /// # Returns
    ///
    /// A Result containing a new instance of [`NostrMlsSqliteStorage`] or an error.
    pub fn new<P: AsRef<Path>>(file_path: P) -> Result<Self, error::Error> {
        // Create or open the SQLite database
        let mls_connection = Connection::open(&file_path)?;

        // Enable foreign keys
        mls_connection.execute_batch("PRAGMA foreign_keys = ON;")?;

        // Create OpenMLS storage
        let mut openmls_storage = openmls_sqlite_storage::SqliteStorageProvider::<
            JsonCodec,
            Connection,
        >::new(mls_connection);

        // Initialize the OpenMLS storage
        if let Err(e) = openmls_storage.initialize() {
            return Err(error::Error::OpenMls(e.to_string()));
        }

        // Create a new connection for the Nostr MLS storage
        let mut nostr_mls_connection = Connection::open(&file_path)?;

        // Enable foreign keys
        nostr_mls_connection.execute_batch("PRAGMA foreign_keys = ON;")?;

        // Apply migrations
        migrations::run_migrations(&mut nostr_mls_connection)?;

        Ok(Self {
            openmls_storage,
            db_connection: Arc::new(Mutex::new(nostr_mls_connection)),
        })
    }

    /// Creates a new in-memory [`NostrMlsSqliteStorage`] for testing purposes.
    ///
    /// # Returns
    ///
    /// A Result containing a new in-memory instance of [`NostrMlsSqliteStorage`] or an error.
    #[cfg(test)]
    pub fn new_in_memory() -> Result<Self, error::Error> {
        // Create an in-memory SQLite database
        let mls_connection = Connection::open_in_memory()?;

        // Enable foreign keys
        mls_connection.execute_batch("PRAGMA foreign_keys = ON;")?;

        // Create OpenMLS storage
        let mut openmls_storage = openmls_sqlite_storage::SqliteStorageProvider::<
            JsonCodec,
            Connection,
        >::new(mls_connection);

        // Initialize the OpenMLS storage
        if let Err(e) = openmls_storage.initialize() {
            return Err(error::Error::OpenMls(e.to_string()));
        }

        // For in-memory databases, we need to share the connection
        // to keep the database alive, so we will clone the connection
        // and let OpenMLS use a new handle
        let mut nostr_mls_connection = Connection::open_in_memory()?;

        // Enable foreign keys
        nostr_mls_connection.execute_batch("PRAGMA foreign_keys = ON;")?;

        // Setup the schema in this connection as well
        migrations::run_migrations(&mut nostr_mls_connection)?;

        Ok(Self {
            openmls_storage,
            db_connection: Arc::new(Mutex::new(nostr_mls_connection)),
        })
    }
}

/// Implementation of [`NostrMlsStorageProvider`] for SQLite-based storage.
impl NostrMlsStorageProvider for NostrMlsSqliteStorage {
    type OpenMlsStorageProvider = MlsStorage;

    /// Returns the backend type.
    ///
    /// # Returns
    ///
    /// [`Backend::SQLite`] indicating this is a SQLite-based storage implementation.
    fn backend(&self) -> Backend {
        Backend::SQLite
    }

    /// Get a reference to the openmls storage provider.
    ///
    /// This method provides access to the underlying OpenMLS storage provider.
    /// This is primarily useful for internal operations and testing.
    ///
    /// # Returns
    ///
    /// A reference to the openmls storage implementation.
    fn openmls_storage(&self) -> &Self::OpenMlsStorageProvider {
        &self.openmls_storage
    }

    /// Get a mutable reference to the openmls storage provider.
    ///
    /// This method provides mutable access to the underlying OpenMLS storage provider.
    /// This is primarily useful for internal operations and testing.
    ///
    /// # Returns
    ///
    /// A mutable reference to the openmls storage implementation.
    fn openmls_storage_mut(&mut self) -> &mut Self::OpenMlsStorageProvider {
        &mut self.openmls_storage
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn test_new_in_memory() {
        let storage = NostrMlsSqliteStorage::new_in_memory();
        assert!(storage.is_ok());
        let storage = storage.unwrap();
        assert_eq!(storage.backend(), Backend::SQLite);
    }

    #[test]
    fn test_backend_type() {
        let storage = NostrMlsSqliteStorage::new_in_memory().unwrap();
        assert_eq!(storage.backend(), Backend::SQLite);
        assert!(storage.backend().is_persistent());
    }

    #[test]
    fn test_file_based_storage() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test_db.sqlite");

        // Create a new storage
        let storage = NostrMlsSqliteStorage::new(&db_path);
        assert!(storage.is_ok());

        // Verify file exists
        assert!(db_path.exists());

        // Create a second instance that connects to the same file
        let storage2 = NostrMlsSqliteStorage::new(&db_path);
        assert!(storage2.is_ok());

        // Clean up
        drop(storage);
        drop(storage2);
        temp_dir.close().unwrap();
    }

    #[test]
    fn test_invalid_path() {
        let invalid_path = "/nonexistent/directory/db.sqlite";
        let storage = NostrMlsSqliteStorage::new(invalid_path);
        assert!(storage.is_err());

        if let Err(err) = storage {
            match err {
                error::Error::Rusqlite(_) => {} // Expected error type
                _ => panic!("Expected Rusqlite error, got {:?}", err),
            }
        }
    }

    #[test]
    fn test_openmls_storage_access() {
        let storage = NostrMlsSqliteStorage::new_in_memory().unwrap();

        // Test that we can get a reference to the openmls storage
        let _openmls_storage = storage.openmls_storage();

        // Test mutable accessor
        let mut mutable_storage = NostrMlsSqliteStorage::new_in_memory().unwrap();
        let _mutable_ref = mutable_storage.openmls_storage_mut();
    }

    #[test]
    fn test_database_tables() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("migration_test.sqlite");

        // Create a new SQLite database
        let storage = NostrMlsSqliteStorage::new(&db_path).unwrap();

        // Verify the database has been properly initialized with migrations
        {
            let conn_guard = storage.db_connection.lock().unwrap();

            // Check if the tables exist
            let mut stmt = conn_guard
                .prepare("SELECT name FROM sqlite_master WHERE type='table'")
                .unwrap();
            let table_names: Vec<String> = stmt
                .query_map([], |row| row.get(0))
                .unwrap()
                .map(|r| r.unwrap())
                .collect();

            // Check for essential tables
            assert!(table_names.contains(&"groups".to_string()));
            assert!(table_names.contains(&"messages".to_string()));
            assert!(table_names.contains(&"welcomes".to_string()));
            assert!(table_names.contains(&"processed_messages".to_string()));
            assert!(table_names.contains(&"processed_welcomes".to_string()));
            assert!(table_names.contains(&"group_relays".to_string()));
        } // conn_guard is dropped here when it goes out of scope

        // Drop explicitly to release all resources
        drop(storage);
        temp_dir.close().unwrap();
    }
}
