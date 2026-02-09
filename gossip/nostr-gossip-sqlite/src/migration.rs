use std::cmp::Ordering;

use rusqlite::Transaction;

use crate::error::{Error, MigrationError};

const DB_VERSION: i64 = 1;

pub(super) fn run(tx: &Transaction<'_>) -> Result<(), Error> {
    // Get the current version
    let mut curr_version: i64 = curr_db_version(tx)?;

    match curr_version.cmp(&DB_VERSION) {
        Ordering::Less => {
            if curr_version == 0 {
                curr_version = mig_init(tx)?;
            }

            // if curr_version == 1 {
            //     curr_version = mig_1_to_2(tx)?;
            // }

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

fn mig_init(tx: &Transaction<'_>) -> Result<i64, Error> {
    tx.execute_batch(include_str!("../migrations/001_init.sql"))?;
    set_db_version(tx, 1)?;
    Ok(1)
}
