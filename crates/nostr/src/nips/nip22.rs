// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP22: Comment
//!
//! <https://github.com/nostr-protocol/nips/blob/master/22.md>

use crate::nips::nip01::Coordinate;
use crate::nips::nip73::ExternalContentId;
use crate::{Alphabet, Event, EventId, Kind, PublicKey, RelayUrl, TagKind, TagStandard, Url};

/// Borrowed comment extracted data
pub enum Comment<'a> {
    /// Event
    Event {
        /// Event ID
        id: &'a EventId,
        /// Relay hint
        relay_hint: Option<&'a RelayUrl>,
        /// Public key hint
        pubkey_hint: Option<&'a PublicKey>,
        /// Kind
        kind: Option<&'a Kind>,
    },
    /// Coordinate
    Coordinate {
        /// Coordinate
        address: &'a Coordinate,
        /// Relay hint
        relay_hint: Option<&'a RelayUrl>,
        /// Kind
        kind: Option<&'a Kind>,
    },
    /// External content
    External {
        /// Content
        content: &'a ExternalContentId,
        /// Web hint
        hint: Option<&'a Url>,
    },
}

/// Extract NIP22 root data
pub fn extract_root(event: &Event) -> Option<Comment> {
    extract_data(event, true)
}

/// Extract NIP22 parent data
pub fn extract_parent(event: &Event) -> Option<Comment> {
    extract_data(event, false)
}

fn extract_data(event: &Event, is_root: bool) -> Option<Comment> {
    if event.kind != Kind::Comment {
        return None;
    }

    // Try to extract event
    if let Some((event_id, relay_hint, public_key)) = extract_event(event, is_root) {
        return Some(Comment::Event {
            id: event_id,
            relay_hint,
            pubkey_hint: public_key,
            kind: extract_kind(event, is_root),
        });
    }

    // Try to extract coordinate
    if let Some((address, relay_hint)) = extract_coordinate(event, is_root) {
        return Some(Comment::Coordinate {
            address,
            relay_hint,
            kind: extract_kind(event, is_root),
        });
    }

    if let Some((content, hint)) = extract_external(event, is_root) {
        return Some(Comment::External { content, hint });
    }

    None
}

fn check_return<T>(val: T, is_root: bool, uppercase: bool) -> Option<T> {
    if (is_root && uppercase) || (!is_root && !uppercase) {
        return Some(val);
    }

    None
}

/// Returns the first kind tag that matches the `is_root` condition.
///
/// # Example:
/// * is_root = true -> returns first `K` tag
/// * is_root = false -> returns first `k` tag
fn extract_kind(event: &Event, is_root: bool) -> Option<&Kind> {
    event
        .tags
        .filter_standardized(TagKind::single_letter(Alphabet::K, is_root))
        .find_map(|tag| match tag {
            TagStandard::Kind { kind, uppercase } => check_return(kind, is_root, *uppercase),
            _ => None,
        })
}

/// Returns the first event tag that matches the `is_root` condition.
///
/// # Example:
/// * is_root = true -> returns first `E` tag
/// * is_root = false -> returns first `e` tag
fn extract_event(
    event: &Event,
    is_root: bool,
) -> Option<(&EventId, Option<&RelayUrl>, Option<&PublicKey>)> {
    event
        .tags
        .filter_standardized(TagKind::single_letter(Alphabet::E, is_root))
        .find_map(|tag| match tag {
            TagStandard::Event {
                event_id,
                relay_url,
                public_key,
                uppercase,
                ..
            } => check_return(
                (event_id, relay_url.as_ref(), public_key.as_ref()),
                is_root,
                *uppercase,
            ),
            _ => None,
        })
}

/// Returns the first coordinate tag that matches the `is_root` condition.
///
/// # Example:
/// * is_root = true -> returns first `A` tag
/// * is_root = false -> returns first `a` tag
fn extract_coordinate(event: &Event, is_root: bool) -> Option<(&Coordinate, Option<&RelayUrl>)> {
    event
        .tags
        .filter_standardized(TagKind::single_letter(Alphabet::A, is_root))
        .find_map(|tag| match tag {
            TagStandard::Coordinate {
                coordinate,
                relay_url,
                uppercase,
                ..
            } => check_return((coordinate, relay_url.as_ref()), is_root, *uppercase),
            _ => None,
        })
}

/// Returns the first external content tag that matches the `is_root` condition.
///
/// # Example:
/// * is_root = true -> returns first `I` tag
/// * is_root = false -> returns first `i` tag
fn extract_external(event: &Event, is_root: bool) -> Option<(&ExternalContentId, Option<&Url>)> {
    event
        .tags
        .filter_standardized(TagKind::single_letter(Alphabet::I, is_root))
        .find_map(|tag| match tag {
            TagStandard::ExternalContent {
                content,
                hint,
                uppercase,
            } => check_return((content, hint.as_ref()), is_root, *uppercase),
            _ => None,
        })
}
