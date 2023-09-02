// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! NIP57
//!
//! <https://github.com/nostr-protocol/nips/blob/master/57.md>

use alloc::string::String;
use alloc::vec::Vec;

use bitcoin::secp256k1::XOnlyPublicKey;

use super::nip33::ParameterizedReplaceableEvent;
use crate::{EventId, UncheckedUrl};

/// Zap Request Data
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ZapRequestData {
    /// Public key of the recipient
    pub public_key: XOnlyPublicKey,
    /// List of relays the recipient's wallet should publish its zap receipt to
    pub relays: Vec<UncheckedUrl>,
    /// Amount in `millisats` the sender intends to pay
    pub amount: Option<u64>,
    /// Lnurl pay url of the recipient, encoded using bech32 with the prefix lnurl.
    pub lnurl: Option<String>,
    /// Event ID
    pub event_id: Option<EventId>,
    /// NIP-33 event coordinate that allows tipping parameterized replaceable events such as NIP-23 long-form notes.
    pub event_coordinate: Option<ParameterizedReplaceableEvent>,
}

impl ZapRequestData {
    /// New Zap Request Data
    pub fn new(public_key: XOnlyPublicKey, relays: Vec<UncheckedUrl>) -> Self {
        Self {
            public_key,
            relays,
            amount: None,
            lnurl: None,
            event_id: None,
            event_coordinate: None,
        }
    }

    /// Amount in `millisats` the sender intends to pay
    pub fn amount(self, amount: u64) -> Self {
        Self {
            amount: Some(amount),
            ..self
        }
    }

    /// Lnurl pay url of the recipient, encoded using bech32 with the prefix lnurl.
    pub fn lnurl<S>(self, lnurl: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            lnurl: Some(lnurl.into()),
            ..self
        }
    }

    /// Event ID
    pub fn event_id(self, event_id: EventId) -> Self {
        Self {
            event_id: Some(event_id),
            ..self
        }
    }

    /// NIP-33 event coordinate that allows tipping parameterized replaceable events such as NIP-23 long-form notes.
    pub fn event_coordinate(self, event_coordinate: ParameterizedReplaceableEvent) -> Self {
        Self {
            event_coordinate: Some(event_coordinate),
            ..self
        }
    }
}
