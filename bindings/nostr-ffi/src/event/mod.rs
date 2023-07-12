// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::ops::Deref;
use std::sync::Arc;

use nostr::Event as EventSdk;

pub mod builder;

use crate::error::Result;
use crate::PublicKey;

pub struct Event {
    event: EventSdk,
}

impl From<EventSdk> for Event {
    fn from(event: EventSdk) -> Self {
        Self { event }
    }
}

impl Deref for Event {
    type Target = EventSdk;
    fn deref(&self) -> &Self::Target {
        &self.event
    }
}

impl Event {
    pub fn pubkey(&self) -> Arc<PublicKey> {
        Arc::new(self.event.pubkey.into())
    }

    pub fn kind(&self) -> u64 {
        self.event.kind.into()
    }

    pub fn content(&self) -> String {
        self.event.content.clone()
    }
}

impl Event {
    pub fn verify(&self) -> bool {
        self.event.verify().is_ok()
    }

    pub fn from_json(json: String) -> Result<Self> {
        Ok(Self {
            event: EventSdk::from_json(json)?,
        })
    }

    pub fn as_json(&self) -> String {
        self.event.as_json()
    }
}
