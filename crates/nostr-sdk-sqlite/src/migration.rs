// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::cmp::Ordering;

use rusqlite::Connection;

use crate::store::{Error, PooledConnection};

/// Latest database version
pub const DB_VERSION: usize = 1;

/// Startup DB Pragmas
pub const STARTUP_SQL: &str = r##"
PRAGMA main.synchronous=NORMAL;
PRAGMA foreign_keys = ON;
PRAGMA journal_size_limit=32768;
pragma mmap_size = 17179869184; -- cap mmap at 16GB
"##;

/// Schema error
#[derive(Debug, thiserror::Error)]
pub enum MigrationError {
    /// Database versione newer than supported
    #[error(
        "Database version is newer than supported by this executable (v{current} > v{DB_VERSION})"
    )]
    NewerDbVersion { current: usize },
}

/// Determine the current application database schema version.
pub fn curr_db_version(conn: &mut Connection) -> Result<usize, Error> {
    let query = "PRAGMA user_version;";
    let curr_version = conn.query_row(query, [], |row| row.get(0))?;
    Ok(curr_version)
}

/// Upgrade DB to latest version, and execute pragma settings
pub fn run(conn: &mut PooledConnection) -> Result<(), Error> {
    // check the version.
    let mut curr_version = curr_db_version(conn)?;
    log::info!("DB version = {:?}", curr_version);

    match curr_version.cmp(&DB_VERSION) {
        // Database is new or not current
        Ordering::Less => {
            // initialize from scratch
            if curr_version == 0 {
                curr_version = mig_init(conn)?;
            }

            // for initialized but out-of-date schemas, proceed to
            // upgrade sequentially until we are current.
            /* if curr_version == 1 {
                curr_version = mig_1_to_2(conn)?;
            } */
            /* if curr_version == 2 {
                curr_version = mig_2_to_3(conn)?;
            } */

            if curr_version == DB_VERSION {
                log::info!("All migration scripts completed successfully (v{DB_VERSION})");
            }
        }
        // Database is current, all is good
        Ordering::Equal => {
            log::debug!("Database version was already current (v{DB_VERSION})");
        }
        // Database is newer than what this code understands, abort
        Ordering::Greater => {
            return Err(Error::Migration(MigrationError::NewerDbVersion {
                current: curr_version,
            }));
        }
    }

    // Setup PRAGMA
    conn.execute_batch(STARTUP_SQL)?;
    log::debug!("SQLite PRAGMA startup completed");
    Ok(())
}

fn mig_init(conn: &mut PooledConnection) -> Result<usize, Error> {
    conn.execute_batch(include_str!("../migrations/001_init.sql"))?;
    log::info!("database schema initialized to v1");
    Ok(1)
}

/* fn mig_1_to_2(conn: &mut PooledConnection) -> Result<usize, Error> {
    conn.execute_batch(include_str!("../migrations/002.sql"))?;
    log::info!("database schema upgraded v1 -> v2");
    Ok(2)
} */
