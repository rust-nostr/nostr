// Copyright (c) 2024 Michael Dilger
// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::collections::HashSet;
use std::fmt;
use std::future::IntoFuture;
use std::io::{self, ErrorKind};
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use futures::executor::block_on;
use indexed_db_futures::js_sys::JsString;
use indexed_db_futures::prelude::OpenDbRequest;
use indexed_db_futures::request::IdbOpenDbRequestLike;
use indexed_db_futures::web_sys::{DomException, IdbTransactionMode};
use indexed_db_futures::{IdbDatabase, IdbQuerySource, IdbVersionChangeEvent};
use nostr::util::hex;
use redb::StorageBackend;
use wasm_bindgen::{JsCast, JsValue};

const CURRENT_DB_VERSION: u32 = 3;
const STORE_NAME: &str = "rust-nostr-redb";
const KEY_NAME: &str = "rust-nostr-redb-key";

/// Error
#[derive(Debug)]
pub enum Error {
    Poison,
    /// DOM error
    DomException {
        /// DomException code
        code: u16,
        /// Specific name of the DomException
        name: String,
        /// Message given to the DomException
        message: String,
    },
    Other(String),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Poison => write!(f, "RwLock poisoned."),
            Self::DomException {
                name,
                code,
                message,
            } => write!(f, "DomException {name} ({code}): {message}"),
            Self::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl From<DomException> for Error {
    fn from(frm: DomException) -> Self {
        Self::DomException {
            name: frm.name(),
            message: frm.message(),
            code: frm.code(),
        }
    }
}

impl From<JsValue> for Error {
    fn from(e: JsValue) -> Self {
        Self::Other(format!("{e:?}"))
    }
}

fn into_io_err(e: Error) -> io::Error {
    io::Error::new(ErrorKind::Other, e)
}

/// Helper struct for upgrading the inner DB.
#[derive(Debug, Clone, Default)]
struct OngoingMigration {
    /// Names of stores to drop.
    drop_stores: HashSet<&'static str>,
    /// Names of stores to create.
    create_stores: HashSet<&'static str>,
}

/// Acts as temporal in-memory database storage.
#[derive(Debug)]
pub struct IndexeddbBackend {
    db: Arc<IdbDatabase>,
    buf: RwLock<Vec<u8>>,
}

unsafe impl Send for IndexeddbBackend {}

unsafe impl Sync for IndexeddbBackend {}

impl IndexeddbBackend {
    fn out_of_range() -> io::Error {
        io::Error::new(ErrorKind::InvalidInput, "Index out-of-range.")
    }
}

impl IndexeddbBackend {
    /// Creates a new, empty memory backend.
    pub async fn open(name: &str) -> Result<Self, Error> {
        let mut db_req: OpenDbRequest = IdbDatabase::open_u32(&name, CURRENT_DB_VERSION)?;
        db_req.set_on_upgrade_needed(Some(
            move |evt: &IdbVersionChangeEvent| -> Result<(), JsValue> {
                let mut old_version: u32 = evt.old_version() as u32;

                tracing::debug!("Database version: {old_version}");

                if old_version <= 1 {
                    let migration = OngoingMigration {
                        create_stores: HashSet::from([STORE_NAME]),
                        ..Default::default()
                    };
                    Self::apply_migration(old_version, CURRENT_DB_VERSION, migration, evt)?;
                    old_version = CURRENT_DB_VERSION;
                }

                if old_version < 3 {
                    let migration = OngoingMigration {
                        create_stores: HashSet::from([STORE_NAME]),
                        ..Default::default()
                    };
                    Self::apply_migration(old_version, CURRENT_DB_VERSION, migration, evt)?;
                    //old_version = CURRENT_DB_VERSION;
                }

                tracing::debug!("Migration completed.");

                Ok(())
            },
        ));

        let mut this = Self {
            db: Arc::new(db_req.into_future().await?),
            buf: RwLock::new(Vec::new()),
        };

        this.read_buf().await?;

        Ok(this)
    }

    fn apply_migration(
        old_version: u32,
        version: u32,
        migration: OngoingMigration,
        evt: &IdbVersionChangeEvent,
    ) -> Result<(), DomException> {
        tracing::debug!("Migrating from v{old_version} to v{version}");

        // Changing the format can only happen in the upgrade procedure
        for store in migration.drop_stores.iter() {
            evt.db().delete_object_store(store)?;
        }
        for store in migration.create_stores.iter() {
            evt.db().create_object_store(store)?;
        }

        Ok(())
    }

    async fn read_buf(&mut self) -> Result<(), Error> {
        tracing::debug!("Reading buffer from database...");

        let tx = self
            .db
            .transaction_on_one_with_mode(STORE_NAME, IdbTransactionMode::Readonly)?;
        let store = tx.object_store(STORE_NAME)?;

        if let Some(jsvalue) = store.get(&JsValue::from_str(KEY_NAME))?.await? {
            if let Some(encoded) = js_value_to_string(jsvalue) {
                tracing::debug!("Found buffer in database. Decoding...");
                let buf = hex::decode(encoded)
                    .map_err(|_| Error::Other("Failed to decode hex string".to_string()))?;
                self.buf = RwLock::new(buf);
                tracing::debug!("Buffer decoded.");
            }
        }

        Ok(())
    }

    /// Gets a read guard for this backend.
    fn read(&self) -> Result<RwLockReadGuard<'_, Vec<u8>>, Error> {
        self.buf.read().map_err(|_| Error::Poison)
    }

    /// Gets a write guard for this backend.
    fn write(&self) -> Result<RwLockWriteGuard<'_, Vec<u8>>, Error> {
        self.buf.write().map_err(|_| Error::Poison)
    }
}

impl StorageBackend for IndexeddbBackend {
    fn len(&self) -> Result<u64, io::Error> {
        Ok(self.read().map_err(into_io_err)?.len() as u64)
    }

    fn read(&self, offset: u64, len: usize) -> Result<Vec<u8>, io::Error> {
        let guard = self.read().map_err(into_io_err)?;
        let offset = usize::try_from(offset).map_err(|_| Self::out_of_range())?;
        if offset + len <= guard.len() {
            Ok(guard[offset..offset + len].to_owned())
        } else {
            Err(Self::out_of_range())
        }
    }

    fn set_len(&self, len: u64) -> Result<(), io::Error> {
        let mut guard = self.write().map_err(into_io_err)?;
        let len = usize::try_from(len).map_err(|_| Self::out_of_range())?;
        if guard.len() < len {
            let additional = len - guard.len();
            guard.reserve(additional);
            for _ in 0..additional {
                guard.push(0);
            }
        } else {
            guard.truncate(len);
        }

        Ok(())
    }

    fn sync_data(&self, _: bool) -> Result<(), io::Error> {
        let guard = self.read().map_err(into_io_err)?;

        let tx = self
            .db
            .transaction_on_one_with_mode(STORE_NAME, IdbTransactionMode::Readwrite)
            .map_err(|e| {
                let e = Error::from(e);
                into_io_err(e)
            })?;
        let store = tx.object_store(STORE_NAME).map_err(|e| {
            let e = Error::from(e);
            into_io_err(e)
        })?;

        // Hex encode
        let encoded: String = hex::encode(guard.as_slice());

        // Store
        let key = JsValue::from_str(KEY_NAME);
        let value = JsValue::from(encoded);
        store.put_key_val(&key, &value).map_err(|e| {
            let e = Error::from(e);
            into_io_err(e)
        })?;

        block_on(async { tx.await.into_result() }).map_err(|e| {
            let e = Error::from(e);
            into_io_err(e)
        })?;

        Ok(())
    }

    fn write(&self, offset: u64, data: &[u8]) -> Result<(), io::Error> {
        let mut guard = self.write().map_err(into_io_err)?;
        let offset = usize::try_from(offset).map_err(|_| Self::out_of_range())?;
        if offset + data.len() <= guard.len() {
            guard[offset..offset + data.len()].copy_from_slice(data);
            Ok(())
        } else {
            Err(Self::out_of_range())
        }
    }
}

fn js_value_to_string(value: JsValue) -> Option<String> {
    let s: JsString = value.dyn_into().ok()?;
    Some(s.into())
}
