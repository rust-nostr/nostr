// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::cmp::Ordering;

use const_format::formatcp;
use rusqlite::Connection;

use crate::error::Error;
use crate::store::PooledConnection;

/// Startup DB Pragmas
pub const STARTUP_SQL: &str = r##"
PRAGMA main.synchronous=NORMAL;
PRAGMA foreign_keys = ON;
PRAGMA journal_size_limit=32768;
pragma mmap_size = 17179869184; -- cap mmap at 16GB
"##;

/// Latest database version
pub const DB_VERSION: usize = 1;

/// Schema definition
const INIT_SQL: &str = formatcp!(
    r##"
-- Database settings
PRAGMA encoding = "UTF-8";
PRAGMA journal_mode=WAL;
PRAGMA main.synchronous=NORMAL;
PRAGMA foreign_keys = ON;
PRAGMA application_id = 1654008667;
PRAGMA user_version = {};

-- Event Table
CREATE TABLE IF NOT EXISTS event (
id TEXT PRIMARY KEY,
pubkey TEXT NOT NULL REFERENCES profile(pubkey),
created_at INTEGER NOT NULL,
kind INTEGER NOT NULL,
tags BLOB NOT NULL,
content TEXT NOT NULL,
sig TEXT NOT NULL
);

-- Event Indexes
CREATE INDEX IF NOT EXISTS event_pubkey_index ON event(pubkey);
CREATE INDEX IF NOT EXISTS created_at_index ON event(created_at);
CREATE INDEX IF NOT EXISTS event_composite_index ON event(kind,created_at);

-- Profile Table
CREATE TABLE IF NOT EXISTS profile (
pubkey TEXT PRIMARY KEY NOT NULL,
name TEXT DEFAULT NULL,
display_name TEXT DEFAULT NULL,
about TEXT DEFAULT NULL,
website TEXT DEFAULT NULL,
picture TEXT DEFAULT NULL,
nip05 TEXT DEFAULT NULL,
lud06 TEXT DEFAULT NULL,
lud16 TEXT DEFAULT NULL,
followed BOOLEAN DEFAULT FALSE,
metadata_at INTEGER DEFAULT 0
);

-- Reactions Table
CREATE TABLE IF NOT EXISTS reaction (
id INTEGER PRIMARY KEY AUTOINCREMENT,
event_id TEXT NOT NULL,
pubkey TEXT NOT NULL REFERENCES profile(pubkey),
content TEXT NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS reaction_index ON reaction(event_id,pubkey,content);

-- Relays Table
CREATE TABLE IF NOT EXISTS relay (
id INTEGER PRIMARY KEY AUTOINCREMENT,
url TEXT NOT NULL,
proxy TEXT DEFAULT NULL,
enabled BOOLEAN DEFAULT TRUE
);

-- Relays Indexes
CREATE UNIQUE INDEX IF NOT EXISTS relay_url_index ON relay(url);

INSERT OR IGNORE INTO relay (url, enabled) values
('wss://relay.damus.io', 1),
('wss://relay.nostr.ch', 1),
('wss://relay.nostr.info', 1),
('wss://relay.nostr.bg', 1),
('wss://nostr.bitcoiner.social', 1),
('wss://nostr.openchain.fr', 1),
('wss://nostr-relay.wlvs.space', 0),
('wss://nostr-pub.semisol.dev', 0),
('wss://nostr.oxtr.dev', 0),
('wss://brb.io', 0),
('wss://relay.grunch.dev', 0),
('wss://nostr-pub.wellorder.net', 0),
('wss://nostr.sandwich.farm', 0),
('wss://nostr.orangepill.dev', 0);
"##,
    DB_VERSION
);

/// Determine the current application database schema version.
pub fn curr_db_version(conn: &mut Connection) -> Result<usize, Error> {
    let query = "PRAGMA user_version;";
    let curr_version = conn.query_row(query, [], |row| row.get(0))?;
    Ok(curr_version)
}

fn mig_init(conn: &mut PooledConnection) -> Result<usize, Error> {
    match conn.execute_batch(INIT_SQL) {
        Ok(()) => {
            log::info!(
                "database pragma/schema initialized to v{}, and ready",
                DB_VERSION
            );
        }
        Err(err) => {
            log::error!("update failed: {}", err);
            panic!("database could not be initialized");
        }
    }
    Ok(DB_VERSION)
}

/// Upgrade DB to latest version, and execute pragma settings
pub fn upgrade_db(conn: &mut PooledConnection) -> Result<(), Error> {
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

            if curr_version == DB_VERSION {
                log::info!(
                    "All migration scripts completed successfully.  Welcome to v{}.",
                    DB_VERSION
                );
            }
        }
        // Database is current, all is good
        Ordering::Equal => {
            log::debug!("Database version was already current (v{})", DB_VERSION);
        }
        // Database is newer than what this code understands, abort
        Ordering::Greater => {
            panic!(
                "Database version is newer than supported by this executable (v{} > v{})",
                curr_version, DB_VERSION
            );
        }
    }

    // Setup PRAGMA
    conn.execute_batch(STARTUP_SQL)?;
    log::debug!("SQLite PRAGMA startup completed");
    Ok(())
}
