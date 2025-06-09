// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Nostr database extension

use std::collections::{BTreeSet, HashMap, HashSet};

use nostr::prelude::*;

use crate::{DatabaseError, Events, NostrDatabase, Profile, RelaysMap};

/// Nostr Event Store Extension
pub trait NostrDatabaseExt: NostrDatabase {
    /// Get public key metadata
    fn metadata(
        &self,
        public_key: PublicKey,
    ) -> BoxedFuture<Result<Option<Metadata>, DatabaseError>> {
        Box::pin(async move {
            let filter = Filter::new()
                .author(public_key)
                .kind(Kind::Metadata)
                .limit(1);
            let events: Events = self.query(filter).await?;
            match events.first_owned() {
                Some(event) => Ok(Some(
                    Metadata::from_json(event.content).map_err(DatabaseError::backend)?,
                )),
                None => Ok(None),
            }
        })
    }

    /// Get contact list public keys
    fn contacts_public_keys(
        &self,
        public_key: PublicKey,
    ) -> BoxedFuture<Result<HashSet<PublicKey>, DatabaseError>> {
        Box::pin(async move {
            let filter = Filter::new()
                .author(public_key)
                .kind(Kind::ContactList)
                .limit(1);
            let events: Events = self.query(filter).await?;
            match events.first_owned() {
                Some(event) => Ok(event.tags.public_keys().copied().collect()),
                None => Ok(HashSet::new()),
            }
        })
    }

    /// Get contact list with metadata of [`PublicKey`]
    fn contacts(
        &self,
        public_key: PublicKey,
    ) -> BoxedFuture<Result<BTreeSet<Profile>, DatabaseError>> {
        Box::pin(async move {
            let filter = Filter::new()
                .author(public_key)
                .kind(Kind::ContactList)
                .limit(1);
            let events: Events = self.query(filter).await?;
            match events.first_owned() {
                Some(event) => {
                    // Get contacts metadata
                    let filter = Filter::new()
                        .authors(event.tags.public_keys().copied())
                        .kind(Kind::Metadata);
                    let mut contacts: HashSet<Profile> = self
                        .query(filter)
                        .await?
                        .into_iter()
                        .map(|e| {
                            let metadata: Metadata =
                                Metadata::from_json(&e.content).unwrap_or_default();
                            Profile::new(e.pubkey, metadata)
                        })
                        .collect();

                    // Extend with missing public keys
                    contacts.extend(event.tags.public_keys().copied().map(Profile::from));

                    Ok(contacts.into_iter().collect())
                }
                None => Ok(BTreeSet::new()),
            }
        })
    }

    /// Get relays list for [PublicKey]
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/65.md>
    fn relay_list(&self, public_key: PublicKey) -> BoxedFuture<Result<RelaysMap, DatabaseError>> {
        Box::pin(async move {
            // Query
            let filter: Filter = Filter::default()
                .author(public_key)
                .kind(Kind::RelayList)
                .limit(1);
            let events: Events = self.query(filter).await?;

            // Extract relay list (NIP65)
            match events.first_owned() {
                Some(event) => Ok(nip65::extract_owned_relay_list(event).collect()),
                None => Ok(HashMap::new()),
            }
        })
    }

    /// Get relays list for public keys
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/65.md>
    fn relay_lists<'a, I>(
        &'a self,
        public_keys: I,
    ) -> BoxedFuture<'a, Result<HashMap<PublicKey, RelaysMap>, DatabaseError>>
    where
        I: IntoIterator<Item = PublicKey> + Send + 'a,
    {
        Box::pin(async move {
            // Query
            let filter: Filter = Filter::default().authors(public_keys).kind(Kind::RelayList);
            let events: Events = self.query(filter).await?;

            let mut map = HashMap::with_capacity(events.len());

            for event in events.into_iter() {
                map.insert(
                    event.pubkey,
                    nip65::extract_owned_relay_list(event).collect(),
                );
            }

            Ok(map)
        })
    }
}

impl<T: NostrDatabase + ?Sized> NostrDatabaseExt for T {}
