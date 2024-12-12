// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::cmp::Ordering;

use nostr::prelude::*;
use nostr_database::flatbuffers::FlatBufferDecodeBorrowed;
use redb::AccessGuard;

use super::filter::DatabaseFilter;
use crate::store::Error;

pub struct AccessGuardEvent<'a> {
    pub guard: AccessGuard<'a, &'static [u8]>,
    pub id: [u8; 32],
    pub created_at: Timestamp,
    pub kind: u16,
}

impl PartialEq for AccessGuardEvent<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for AccessGuardEvent<'_> {}

impl PartialOrd for AccessGuardEvent<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for AccessGuardEvent<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.created_at != other.created_at {
            // Descending order
            // NOT EDIT, will break many things!!
            self.created_at.cmp(&other.created_at).reverse()
        } else {
            self.id.cmp(&other.id)
        }
    }
}

impl<'a> AccessGuardEvent<'a> {
    pub fn new(guard: AccessGuard<'a, &'static [u8]>) -> Result<Self, Error> {
        let value = guard.value();
        let event = EventBorrow::decode(value)?;
        let id = *event.id;
        let created_at = event.created_at;
        let kind = event.kind;
        Ok(Self {
            guard,
            id,
            created_at,
            kind,
        })
    }

    pub fn match_filter(&self, filter: &DatabaseFilter) -> Result<bool, Error> {
        let value = self.guard.value();
        let event = EventBorrow::decode(value)?;
        Ok(filter.match_event(&event))
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn to_event(self) -> Result<Event, Error> {
        let value = self.guard.value();
        let temp = EventBorrow::decode(value)?;
        Ok(temp.into_owned())
    }
}
