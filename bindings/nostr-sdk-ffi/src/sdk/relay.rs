// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::ops::Deref;

use anyhow::Result;
use nostr_sdk::relay::RelayPoolNotifications as RelayPoolNotificationsSdk;

use crate::base::event::Event;
use crate::FromResult;

pub enum RelayPoolNotifications {
    ReceivedEvent { json: String },
    RelayDisconnected { url: String },
}

impl FromResult<RelayPoolNotificationsSdk> for RelayPoolNotifications {
    fn from_result(f: RelayPoolNotificationsSdk) -> Result<Self> {
        Ok(match f {
            RelayPoolNotificationsSdk::ReceivedEvent(event) => Self::ReceivedEvent {
                json: event.as_json(),
            },
            RelayPoolNotificationsSdk::RelayDisconnected(url) => Self::RelayDisconnected { url },
        })
    }
}

impl FromResult<RelayPoolNotifications> for RelayPoolNotificationsSdk {
    fn from_result(f: RelayPoolNotifications) -> Result<Self> {
        Ok(match f {
            RelayPoolNotifications::ReceivedEvent { json } => {
                Self::ReceivedEvent(Event::new_from_json(json)?.deref().clone())
            }
            RelayPoolNotifications::RelayDisconnected { url } => Self::RelayDisconnected(url),
        })
    }
}
