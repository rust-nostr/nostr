// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Accounts.dat format
//!
//! Fields:
//! * Version: 4 bytes, little-endian
//! * Accounts count: variable
//! * Accounts: variable
//!
//! Account format:
//! * Identifier bit (ex. 0x01)
//! * Name: variable (TLV)
//!     * Identifier (0x02)
//!     * Len: u8
//!     * Value: variable
//! * Key: variable (TLV)

use std::collections::BTreeSet;
use std::iter::Skip;
use std::slice::Iter;

use nostr::prelude::*;

use super::constants::ACCOUNT;
use super::Version;

pub struct NostrKeyringDat {
    version: Version,
    list: BTreeSet<Account>,
}

impl NostrKeyringDat {
    pub fn parse(slice: &[u8]) -> Self {
        // Get version
        // TODO

        let mut iter: Skip<Iter<u8>> = slice.iter().skip(4);

        // TODO: match version

        // V1
        // Get number of accounts
        // Start iterating and parsing accounts
        //let account = Account::parse(slice, version)?;

        todo!()
    }
}

pub struct Account {
    name: String,
    key: AccountKey,
}

impl Account {
    fn parse(slice: &mut [u8], version: Version) -> Self {
        match version {
            Version::V1 => {
                // Get identifier
                let identifier: u8 = slice.first().copied().unwrap();

                if identifier == ACCOUNT {
                    // TODO: parse name and key
                }

                todo!()
            }
        }
    }
}

pub enum AccountKey {
    Unencrypted(SecretKey),
    Encrypted(EncryptedSecretKey),
    WatchOnly(PublicKey),
}
