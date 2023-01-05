// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::str::FromStr;

use nostr::secp256k1::XOnlyPublicKey;
use nostr::Contact;

use crate::error::Error;
use crate::model::Profile;
use crate::store::Store;

impl Store {
    pub fn insert_profile(&self, profile: Profile) -> Result<(), Error> {
        let conn = self.pool.get()?;
        conn.execute(
            "INSERT INTO profile (pubkey, name, display_name, about, website, picture, nip05, lud06, lud16, followed, metadata_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?);",
            (
                profile.pubkey.to_string(),
                profile.name,
                profile.display_name,
                profile.about,
                profile.website,
                profile.picture,
                profile.nip05,
                profile.lud06,
                profile.lud16,
                if profile.pubkey == self.owner_pubkey {true} else {profile.followed},
                profile.metadata_at,
            ),
        )?;

        Ok(())
    }

    pub fn update_profile(&self, profile: Profile) -> Result<(), Error> {
        let conn = self.pool.get()?;
        let sql: &str = "UPDATE profile SET name = ?, display_name =? , about = ?, website = ?, picture = ?, nip05 = ?, lud06 = ?, lud16 = ?, followed = ?, metadata_at = ? WHERE pubkey = ?;";
        conn.execute(
            sql,
            (
                profile.name,
                profile.display_name,
                profile.about,
                profile.website,
                profile.picture,
                profile.nip05,
                profile.lud06,
                profile.lud16,
                if profile.pubkey == self.owner_pubkey {
                    true
                } else {
                    profile.followed
                },
                profile.metadata_at,
                profile.pubkey.to_string(),
            ),
        )?;

        Ok(())
    }

    pub fn get_profile(&self, public_key: XOnlyPublicKey) -> Result<Profile, Error> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("SELECT * FROM profile WHERE pubkey = ?")?;
        let mut rows = stmt.query([public_key.to_string()])?;

        match rows.next()? {
            Some(row) => {
                let pubkey: String = row.get(0)?;
                Ok(Profile {
                    pubkey: XOnlyPublicKey::from_str(&pubkey)?,
                    name: row.get(1)?,
                    display_name: row.get(2)?,
                    about: row.get(3)?,
                    website: row.get(4)?,
                    picture: row.get(5)?,
                    nip05: row.get(6)?,
                    lud06: row.get(7)?,
                    lud16: row.get(8)?,
                    followed: row.get(9)?,
                    metadata_at: row.get(10)?,
                })
            }
            None => Err(Error::ValueNotFound),
        }
    }

    pub fn set_contacts(&self, list: Vec<Contact>) -> Result<(), Error> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("INSERT INTO profile (pubkey, followed) VALUES(?, ?) ON CONFLICT(pubkey) DO UPDATE SET followed=excluded.followed;")?;
        for contact in list.into_iter() {
            stmt.execute((contact.pk.to_string(), true))?;
        }
        // TODO: update filters with new pubkeys
        Ok(())
    }

    /// Get contacts
    pub fn get_contacts(&self) -> Result<Vec<Profile>, Error> {
        let conn = self.pool.get()?;
        let mut stmt =
            conn.prepare("SELECT * FROM profile WHERE followed = ? ORDER BY name ASC")?;
        let mut rows = stmt.query([true])?;

        let mut profiles = Vec::new();

        while let Ok(Some(row)) = rows.next() {
            let pubkey: String = row.get(0)?;
            profiles.push(Profile {
                pubkey: XOnlyPublicKey::from_str(&pubkey)?,
                name: row.get(1)?,
                display_name: row.get(2)?,
                about: row.get(3)?,
                website: row.get(4)?,
                picture: row.get(5)?,
                nip05: row.get(6)?,
                lud06: row.get(7)?,
                lud16: row.get(8)?,
                followed: row.get(9)?,
                metadata_at: row.get(10)?,
            })
        }

        Ok(profiles)
    }

    pub fn get_contacts_pubkeys(&self) -> Result<Vec<XOnlyPublicKey>, Error> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("SELECT pubkey FROM profile WHERE followed = ?")?;
        let mut rows = stmt.query([true])?;

        let mut authors: Vec<XOnlyPublicKey> = Vec::new();

        while let Ok(Some(row)) = rows.next() {
            let pubkey: String = row.get(0)?;
            authors.push(XOnlyPublicKey::from_str(&pubkey)?);
        }

        if !authors.contains(&self.owner_pubkey) {
            authors.push(self.owner_pubkey);
        }

        Ok(authors)
    }

    /// Get all pubkeys seen
    pub fn get_authors(&self) -> Result<Vec<XOnlyPublicKey>, Error> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("SELECT pubkey FROM profile")?;
        let mut rows = stmt.query([])?;

        let mut authors: Vec<XOnlyPublicKey> = Vec::new();

        while let Ok(Some(row)) = rows.next() {
            let pubkey: String = row.get(0)?;
            authors.push(XOnlyPublicKey::from_str(&pubkey)?);
        }

        if !authors.contains(&self.owner_pubkey) {
            authors.push(self.owner_pubkey);
        }

        Ok(authors)
    }
}
