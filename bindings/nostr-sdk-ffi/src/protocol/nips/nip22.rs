// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;
use std::sync::Arc;

use nostr::nips::nip22::{self, Comment};
use uniffi::Enum;

use super::nip01::Coordinate;
use super::nip73::ExternalContentId;
use crate::protocol::event::{Event, EventId, Kind};
use crate::protocol::key::PublicKey;

/// Extracted NIP22 comment
///
/// <https://github.com/nostr-protocol/nips/blob/master/22.md>
#[derive(Enum)]
pub enum ExtractedComment {
    /// Event
    Event {
        /// Event ID
        id: Arc<EventId>,
        /// Relay hint
        relay_hint: Option<String>,
        /// Public key hint
        pubkey_hint: Option<Arc<PublicKey>>,
        /// Kind
        kind: Option<Arc<Kind>>,
    },
    /// Coordinate
    // NOTE: the enum variant can't have the same name of types in inner fields, otherwise will create issues with kotlin,
    // so rename this to `Address`.
    Address {
        /// Coordinate
        address: Arc<Coordinate>,
        /// Relay hint
        relay_hint: Option<String>,
        /// Kind
        kind: Option<Arc<Kind>>,
    },
    /// External content
    External {
        /// Content
        content: ExternalContentId,
        /// Web hint
        hint: Option<String>,
    },
}

impl From<Comment<'_>> for ExtractedComment {
    fn from(comment: Comment<'_>) -> Self {
        match comment {
            Comment::Event {
                id,
                relay_hint,
                pubkey_hint,
                kind,
            } => Self::Event {
                id: Arc::new((*id).into()),
                relay_hint: relay_hint.map(|u| u.to_string()),
                pubkey_hint: pubkey_hint.map(|p| Arc::new((*p).into())),
                kind: kind.map(|k| Arc::new((*k).into())),
            },
            Comment::Coordinate {
                address,
                relay_hint,
                kind,
            } => Self::Address {
                address: Arc::new(address.clone().into()),
                relay_hint: relay_hint.map(|u| u.to_string()),
                kind: kind.map(|k| Arc::new((*k).into())),
            },
            Comment::External { content, hint } => Self::External {
                content: content.clone().into(),
                hint: hint.map(|u| u.to_string()),
            },
        }
    }
}

/// Extract NIP22 root comment data
pub fn nip22_extract_root(event: &Event) -> Option<ExtractedComment> {
    nip22::extract_root(event.deref()).map(|c| c.into())
}

/// Extract NIP22 parent comment data
pub fn nip22_extract_parent(event: &Event) -> Option<ExtractedComment> {
    nip22::extract_parent(event.deref()).map(|c| c.into())
}
