// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use nostr::Event;

use crate::error::Error;
use crate::store::Store;

impl Store {
    pub fn insert_reaction(&self, event: &Event) -> Result<(), Error> {
        let conn = self.pool.get()?;
        conn.execute(
            "INSERT OR IGNORE INTO reaction (event_id, pubkey, content) VALUES (?, ?, ?);",
            (
                event.id.to_string(),
                event.pubkey.to_string(),
                event.content.clone(),
            ),
        )?;
        Ok(())
    }
}
