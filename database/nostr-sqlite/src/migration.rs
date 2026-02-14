use std::cmp::Ordering;

use rusqlite::{params, OptionalExtension, Transaction};

use crate::error::{Error, MigrationError};

const DB_VERSION: i64 = 2;

pub(super) fn run(tx: &Transaction<'_>) -> Result<(), Error> {
    migrate_sqlx_tables(tx)?;

    // Get the current version
    let mut curr_version: i64 = curr_db_version(tx)?;

    if curr_version == 0 {
        let inferred: i64 = infer_version_from_schema(tx)?;
        if inferred > 0 {
            set_db_version(tx, inferred)?;
            curr_version = inferred;
        }
    }

    match curr_version.cmp(&DB_VERSION) {
        Ordering::Less => {
            if curr_version == 0 {
                curr_version = mig_init(tx)?;
            }

            if curr_version == 1 {
                curr_version = mig_1_to_2(tx)?;
            }

            let _ = curr_version;
        }
        Ordering::Equal => {}
        Ordering::Greater => {
            return Err(MigrationError::NewerVersion {
                current: curr_version,
                supported: DB_VERSION,
            }
            .into());
        }
    }

    Ok(())
}

fn curr_db_version(tx: &Transaction<'_>) -> Result<i64, Error> {
    let version: i64 = tx.query_row("PRAGMA user_version", [], |row| row.get(0))?;
    Ok(version)
}

fn set_db_version(tx: &Transaction<'_>, version: i64) -> Result<(), Error> {
    tx.pragma_update(None, "user_version", version)?;
    Ok(())
}

fn table_exists(tx: &Transaction<'_>, name: &str) -> Result<bool, Error> {
    let exists: Option<String> = tx
        .query_row(
            "SELECT name FROM sqlite_master WHERE type='table' AND name=?1",
            params![name],
            |row| row.get(0),
        )
        .optional()?;
    Ok(exists.is_some())
}

fn migrate_sqlx_tables(tx: &Transaction<'_>) -> Result<(), Error> {
    if !table_exists(tx, "_sqlx_migrations")? {
        return Ok(());
    }

    let dirty: Option<i64> = tx
        .query_row(
            "SELECT version FROM _sqlx_migrations WHERE success = false ORDER BY version LIMIT 1",
            [],
            |row| row.get(0),
        )
        .optional()?;

    if let Some(version) = dirty {
        return Err(MigrationError::Dirty(version).into());
    }

    let version: Option<i64> = tx.query_row(
        "SELECT MAX(version) FROM _sqlx_migrations WHERE success = true",
        [],
        |row| row.get(0),
    )?;
    let version: i64 = version.unwrap_or(0);

    let current_version: i64 = curr_db_version(tx)?;
    let target_version: i64 = current_version.max(version);
    if target_version != current_version {
        set_db_version(tx, target_version)?;
    }

    tx.execute("DROP TABLE _sqlx_migrations", [])?;

    Ok(())
}

fn infer_version_from_schema(tx: &Transaction<'_>) -> Result<i64, Error> {
    if !table_exists(tx, "events")? {
        return Ok(0);
    }

    if table_exists(tx, "vanished_public_keys")? {
        return Ok(2);
    }

    Ok(1)
}

fn mig_init(tx: &Transaction<'_>) -> Result<i64, Error> {
    tx.execute_batch(include_str!("../migrations/001_init.sql"))?;
    set_db_version(tx, 1)?;
    Ok(1)
}

fn mig_1_to_2(tx: &Transaction<'_>) -> Result<i64, Error> {
    tx.execute_batch(include_str!("../migrations/002_vanished_public_keys.sql"))?;
    set_db_version(tx, 2)?;
    Ok(2)
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;

    use rusqlite::Connection;
    use tempfile::TempDir;

    use super::*;
    use crate::store::NostrSqlite;

    #[tokio::test]
    async fn test_remove_sqlx_migrations() {
        // Copy the database in a temp dir
        let temp: TempDir = TempDir::new().unwrap();
        let db_path: PathBuf = {
            let sqlx_db: &[u8] = include_bytes!("../fixtures/nostr-sqlx.sqlite");

            let path: PathBuf = temp.path().join("temp.db");

            let mut db = File::create(&path).unwrap();
            db.write_all(sqlx_db).unwrap();

            path
        };

        // Make sure it's a SQLx database
        {
            let mut conn = Connection::open(&db_path).unwrap();
            let tx = conn.transaction().unwrap();
            assert!(table_exists(&tx, "_sqlx_migrations").unwrap()); // At this stage the _sqlx_migrations must exists
            assert!(table_exists(&tx, "events").unwrap());
            assert!(table_exists(&tx, "vanished_public_keys").unwrap());
        }

        // Run migrations
        {
            let _db = NostrSqlite::open(&db_path).await.unwrap();
        }

        let mut conn = Connection::open(&db_path).unwrap();

        // New version must be the DB_VERSION
        let version: i64 = conn
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .unwrap();
        assert_eq!(version, DB_VERSION);

        // _sqlx_migrations table must be removed, while the others are still there
        let tx = conn.transaction().unwrap();
        assert!(!table_exists(&tx, "_sqlx_migrations").unwrap());
        assert!(table_exists(&tx, "events").unwrap());
        assert!(table_exists(&tx, "vanished_public_keys").unwrap());
    }
}
