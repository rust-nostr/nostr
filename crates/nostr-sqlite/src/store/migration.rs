// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::cmp::Ordering;

use rusqlite::Connection;

use super::{Error, Pool};

/// Latest database version
pub const DB_VERSION: usize = 1;

/// Startup DB Pragmas
pub const STARTUP_SQL: &str = r##"
PRAGMA main.synchronous=NORMAL;
PRAGMA foreign_keys = ON;
PRAGMA journal_size_limit=32768;
pragma mmap_size = 17179869184; -- cap mmap at 16GB
"##;

/// Determine the current application database schema version.
pub fn curr_db_version(conn: &mut Connection) -> Result<usize, Error> {
    let query = "PRAGMA user_version;";
    let curr_version = conn.query_row(query, [], |row| row.get(0))?;
    Ok(curr_version)
}

/// Upgrade DB to latest version, and execute pragma settings
pub(crate) async fn run(pool: &Pool) -> Result<(), Error> {
    pool.interact(|conn| {
        // check the version.
        let mut curr_version = curr_db_version(conn)?;
        tracing::info!("DB version = {:?}", curr_version);

        match curr_version.cmp(&DB_VERSION) {
            // Database is new or not current
            Ordering::Less => {
                // initialize from scratch
                if curr_version == 0 {
                    curr_version = mig_init(conn)?;
                }

                // for initialized but out-of-date schemas, proceed to
                // upgrade sequentially until we are current.
                // if curr_version == 1 {
                // curr_version = mig_1_to_2(conn)?;
                // }
                //
                // if curr_version == 2 {
                // curr_version = mig_2_to_3(conn)?;
                // }
                //
                // if curr_version == 3 {
                // curr_version = mig_3_to_4(conn)?;
                // }
                //
                // if curr_version == 4 {
                // curr_version = mig_4_to_5(conn)?;
                // }
                //
                // if curr_version == 5 {
                // curr_version = mig_5_to_6(conn)?;
                // }
                //
                // if curr_version == 6 {
                // curr_version = mig_6_to_7(conn)?;
                // }

                if curr_version == DB_VERSION {
                    tracing::info!("All migration scripts completed successfully (v{DB_VERSION})");
                }
            }
            // Database is current, all is good
            Ordering::Equal => {
                tracing::debug!("Database version was already current (v{DB_VERSION})");
            }
            // Database is newer than what this code understands, abort
            Ordering::Greater => {
                return Err(Error::NewerDbVersion {
                    current: curr_version,
                    other: DB_VERSION,
                });
            }
        }

        // Setup PRAGMA
        conn.execute_batch(STARTUP_SQL)?;
        tracing::debug!("SQLite PRAGMA startup completed");
        Ok(())
    })
    .await?
}

fn mig_init(conn: &mut Connection) -> Result<usize, Error> {
    conn.execute_batch(include_str!("../../migrations/001_init.sql"))?;
    tracing::info!("database schema initialized to v1");
    Ok(1)
}

// fn mig_1_to_2(conn: &mut Connection) -> Result<usize, Error> {
// conn.execute_batch(include_str!("../../migrations/002_notifications.sql"))?;
// tracing::info!("database schema upgraded v1 -> v2");
// Ok(2)
// }
