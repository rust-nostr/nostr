// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::collections::HashSet;

use nostr::nips::nip65::RelayMetadata;
use nostr::nips::{nip17, nip65};
use nostr::{Event, Kind, PublicKey, RelayUrl, Timestamp};
use rusqlite::{Connection, OptionalExtension, Rows, Transaction};

pub(super) mod error;
mod migrate;

use self::error::Error;
use crate::constant::MAX_RELAYS_LIST;

pub(super) struct ListTimestamps {
    pub(super) created_at: Timestamp,
    pub(super) last_check: Timestamp,
}

#[derive(Debug)]
pub struct Store {
    conn: Connection,
}

impl Store {
    #[inline]
    pub fn new(conn: Connection) -> Self {
        Self { conn }
    }

    #[inline]
    pub(super) fn migrate(&self) -> Result<(), Error> {
        migrate::run(&self.conn)
    }

    pub(super) fn get_pkid_and_listid(
        &mut self,
        pk: &PublicKey,
        kind: Kind,
    ) -> Result<Option<(u64, u64)>, Error> {
        let tx = self.conn.transaction()?;

        let mut res: Option<(u64, u64)> = None;

        if let Some(pkid) = get_pkid(&tx, pk)? {
            if let Some(listid) = get_list_id(&tx, pkid, kind)? {
                res = Some((pkid, listid));
            }
        }

        tx.commit()?;

        Ok(res)
    }

    pub(super) fn update_last_check(
        &mut self,
        public_key: &PublicKey,
        kinds: &[Kind],
        last_check: &Timestamp,
    ) -> Result<(), Error> {
        let tx = self.conn.transaction()?;

        insert_public_key(&tx, public_key)?;
        let pkid: u64 = get_pkid(&tx, public_key)?.expect("The public key ID must exist");

        {
            let mut stmt =
                tx.prepare_cached("UPDATE lists SET last_check = ?1 WHERE pkid=?2 AND kind=?3")?;
            for kind in kinds {
                stmt.execute((last_check.as_u64(), pkid, kind.as_u16()))?;
            }
        }

        tx.commit()?;

        Ok(())
    }

    pub(super) fn get_timestamps(
        &mut self,
        public_key: &PublicKey,
        kind: Kind,
    ) -> Result<ListTimestamps, Error> {
        let tx = self.conn.transaction()?;

        insert_public_key(&tx, public_key)?;
        let pkid: u64 = get_pkid(&tx, public_key)?.expect("The public key ID must exist");

        insert_list(&tx, pkid, kind)?;
        let listid: u64 = get_list_id(&tx, pkid, kind)?.expect("The list ID must exist");

        let (created_at, last_check): (u64, u64) = {
            let mut stmt = tx.prepare_cached(
                "SELECT created_at, last_check FROM lists WHERE id=?1 AND pkid=?2",
            )?;
            stmt.query_row([listid, pkid], |row| Ok((row.get(0)?, row.get(1)?)))?
        };

        tx.commit()?;

        Ok(ListTimestamps {
            created_at: Timestamp::from_secs(created_at),
            last_check: Timestamp::from_secs(last_check),
        })
    }

    /// Get relay URLs related to the list
    pub(super) fn get_relays_url(
        &self,
        pkid: u64,
        list_id: u64,
    ) -> Result<HashSet<RelayUrl>, Error> {
        let mut stmt = self.conn.prepare_cached("SELECT r.relay_url FROM relays_by_list AS rbl JOIN relays AS r ON r.id = rbl.relayid WHERE rbl.pkid = ?1 AND rbl.listid = ?2;")?;
        let rows = stmt.query([pkid, list_id])?;
        extract_relay_urls(rows)
    }

    pub(super) fn get_nip65_relays_url_by_metadata(
        &self,
        pkid: u64,
        list_id: u64,
        metadata: RelayMetadata,
    ) -> Result<HashSet<RelayUrl>, Error> {
        let mut stmt = self.conn.prepare_cached("SELECT r.relay_url FROM relays_by_list AS rbl JOIN relays AS r ON r.id = rbl.relayid WHERE rbl.pkid = ?1 AND rbl.listid = ?2 AND (metadata = ?3 OR metadata IS NULL);")?;
        let rows = stmt.query((pkid, list_id, metadata.as_str()))?;
        extract_relay_urls(rows)
    }

    pub(super) fn process_event(&mut self, event: &Event) -> Result<(), Error> {
        // Begin a transaction on the underlying connection
        let tx = self.conn.transaction()?;

        insert_public_key(&tx, &event.pubkey)?;
        let pkid: u64 = get_pkid(&tx, &event.pubkey)?.expect("The public key ID must exist");

        insert_list(&tx, pkid, event.kind)?;
        let listid: u64 = get_list_id(&tx, pkid, event.kind)?.expect("The list ID must exist");

        let created_at: Timestamp = get_created_at(&tx, pkid, event.kind)?;

        // Check if can update
        if created_at > event.created_at {
            return Ok(());
        }

        // Delete relays
        delete_relays_by_list(&tx, pkid, listid)?;

        match event.kind {
            Kind::RelayList => {
                let iter = nip65::extract_relay_list(event).take(MAX_RELAYS_LIST);

                for (relay_url, metadata) in iter {
                    insert_relay(&tx, relay_url)?;
                    let relayid: u64 = get_relay_id(&tx, relay_url)?;

                    insert_relay_for_list(&tx, pkid, listid, relayid, *metadata)?;
                }
            }
            Kind::InboxRelays => {
                let iter = nip17::extract_relay_list(event).take(MAX_RELAYS_LIST);

                for relay_url in iter {
                    insert_relay(&tx, relay_url)?;
                    let relayid: u64 = get_relay_id(&tx, relay_url)?;

                    insert_relay_for_list(&tx, pkid, listid, relayid, None)?;
                }
            }
            _ => {}
        }

        update_created_at(&tx, pkid, event.kind, &event.created_at)?;

        // Commit
        tx.commit()?;

        Ok(())
    }
}

fn insert_public_key(tx: &Transaction, public_key: &PublicKey) -> Result<(), Error> {
    let mut stmt = tx.prepare_cached("INSERT OR IGNORE INTO users (public_key) VALUES (?1)")?;
    stmt.execute([public_key.as_bytes()])?;
    Ok(())
}

/// Get public key ID
fn get_pkid(tx: &Transaction, public_key: &PublicKey) -> Result<Option<u64>, Error> {
    let mut stmt = tx.prepare_cached("SELECT id FROM users WHERE public_key=?1")?;
    let row_id: Option<u64> = stmt
        .query_row([public_key.as_bytes()], |row| row.get(0))
        .optional()?;
    Ok(row_id)
}

fn insert_relay(tx: &Transaction, relay_url: &RelayUrl) -> Result<(), Error> {
    let mut stmt = tx.prepare_cached("INSERT OR IGNORE INTO relays (relay_url) VALUES (?1)")?;
    stmt.execute([relay_url.as_str_without_trailing_slash()])?;
    Ok(())
}

/// Get relay ID
fn get_relay_id(tx: &Transaction, relay_url: &RelayUrl) -> Result<u64, Error> {
    let mut stmt = tx.prepare_cached("SELECT id FROM relays WHERE relay_url=?1")?;
    let row_id: u64 = stmt.query_row([relay_url.as_str_without_trailing_slash()], |row| {
        row.get(0)
    })?;
    Ok(row_id)
}

fn insert_list(tx: &Transaction, pkid: u64, kind: Kind) -> Result<(), Error> {
    let mut stmt = tx.prepare_cached("INSERT OR IGNORE INTO lists (pkid, kind) VALUES (?1, ?2)")?;
    stmt.execute((pkid, kind.as_u16()))?;
    Ok(())
}

/// Get list ID
fn get_list_id(tx: &Transaction, pkid: u64, kind: Kind) -> Result<Option<u64>, Error> {
    let mut stmt = tx.prepare_cached("SELECT id FROM lists WHERE pkid=?1 AND kind=?2")?;
    let row_id: Option<u64> = stmt
        .query_row((pkid, kind.as_u16()), |row| row.get(0))
        .optional()?;
    Ok(row_id)
}

fn get_created_at(tx: &Transaction, pkid: u64, kind: Kind) -> Result<Timestamp, Error> {
    let mut stmt = tx.prepare_cached("SELECT created_at FROM lists WHERE pkid=?1 AND kind=?2")?;
    let timestamp: u64 = stmt.query_row((pkid, kind.as_u16()), |row| row.get(0))?;
    Ok(Timestamp::from_secs(timestamp))
}

fn update_created_at(
    tx: &Transaction,
    pkid: u64,
    kind: Kind,
    created_at: &Timestamp,
) -> Result<(), Error> {
    let mut stmt =
        tx.prepare_cached("UPDATE lists SET created_at = ?1 WHERE pkid=?2 AND kind=?3")?;
    stmt.execute((created_at.as_u64(), pkid, kind.as_u16()))?;
    Ok(())
}

fn insert_relay_for_list(
    tx: &Transaction,
    pkid: u64,
    list_id: u64,
    relay_id: u64,
    metadata: Option<RelayMetadata>,
) -> Result<(), Error> {
    let metadata: Option<&str> = metadata.as_ref().map(|m| m.as_str());

    let mut stmt = tx.prepare_cached(
        "INSERT INTO relays_by_list (pkid, listid, relayid, metadata) VALUES (?1, ?2, ?3, ?4)",
    )?;
    stmt.execute((pkid, list_id, relay_id, metadata))?;

    Ok(())
}

fn delete_relays_by_list(tx: &Transaction, pkid: u64, list_id: u64) -> Result<(), Error> {
    let mut stmt = tx.prepare_cached("DELETE FROM relays_by_list WHERE pkid=?1 AND listid=?2")?;
    stmt.execute((pkid, list_id))?;
    Ok(())
}

fn extract_relay_urls(mut rows: Rows) -> Result<HashSet<RelayUrl>, Error> {
    let mut relays = HashSet::new();

    while let Some(row) = rows.next()? {
        let relay_url: &str = row.get_ref(0)?.as_str()?;
        let relay_url: RelayUrl = RelayUrl::parse(relay_url)?;
        relays.insert(relay_url);
    }

    Ok(relays)
}
