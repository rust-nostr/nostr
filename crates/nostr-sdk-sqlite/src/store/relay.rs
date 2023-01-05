// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::net::SocketAddr;

use nostr::url::Url;

use crate::error::Error;
use crate::store::Store;

impl Store {
    pub fn insert_relay(&self, url: Url, proxy: Option<SocketAddr>) -> Result<(), Error> {
        let conn = self.pool.get()?;
        conn.execute(
            "INSERT OR IGNORE INTO relay (url, proxy) VALUES (?, ?);",
            (url, proxy.map(|a| a.to_string())),
        )?;
        Ok(())
    }

    pub fn get_relays(&self, enabled: bool) -> Result<Vec<(Url, Option<SocketAddr>)>, Error> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("SELECT url, proxy FROM relay WHERE enabled = ?")?;
        let mut rows = stmt.query([enabled])?;

        let mut relays: Vec<(Url, Option<SocketAddr>)> = Vec::new();
        while let Ok(Some(row)) = rows.next() {
            let url: Url = row.get(0)?;
            let proxy: Option<String> = row.get(1)?;
            relays.push((
                url,
                proxy
                    .map(|p| p.parse())
                    .filter(|r| r.is_ok())
                    .map(|r| r.unwrap()),
            ));
        }
        Ok(relays)
    }

    pub fn delete_relay(&self, url: Url) -> Result<(), Error> {
        let conn = self.pool.get()?;
        conn.execute("DELETE FROM relay WHERE url = ?;", [url])?;
        Ok(())
    }

    pub fn enable_relay(&self, url: Url) -> Result<(), Error> {
        let conn = self.pool.get()?;
        conn.execute("UPDATE relay SET enabled = ? WHERE url = ?;", (1, url))?;
        Ok(())
    }

    pub fn disable_relay(&self, url: Url) -> Result<(), Error> {
        let conn = self.pool.get()?;
        conn.execute("UPDATE relay SET enabled = ? WHERE url = ?;", (0, url))?;
        Ok(())
    }
}
