// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP22: Comment
//!
//! <https://github.com/nostr-protocol/nips/blob/master/22.md>

use alloc::vec::Vec;

use crate::nips::nip01::CoordinateBorrow;
use crate::nips::nip73::ExternalContentId;
use crate::{Alphabet, Event, EventId, Kind, PublicKey, RelayUrl, Tag, TagKind, TagStandard, Url};

#[allow(missing_docs)]
#[deprecated(since = "0.42.0", note = "Use `CommentTarget` instead")]
pub type Comment<'a> = CommentTarget<'a>;

/// Comment target
pub enum CommentTarget<'a> {
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
        address: CoordinateBorrow<'a>,
        /// Relay hint
        relay_hint: Option<&'a RelayUrl>,
        /// Kind
        #[deprecated(since = "0.44.0", note = "Use `address.kind` instead")]
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

impl<'a> CommentTarget<'a> {
    /// Creates a new [`CommentTarget`] pointing to a specific event.
    #[inline]
    pub fn event(
        event_id: &'a EventId,
        kind: &'a Kind,
        author: Option<&'a PublicKey>,
        relay_hint: Option<&'a RelayUrl>,
    ) -> Self {
        Self::Event {
            relay_hint,
            id: event_id,
            pubkey_hint: author,
            kind: Some(kind),
        }
    }

    /// Create a new [`CommentTarget`] pointing to a specific coordinate.
    #[inline]
    pub fn coordinate(coordinate: CoordinateBorrow<'a>, relay_hint: Option<&'a RelayUrl>) -> Self {
        Self::Coordinate {
            address: coordinate,
            relay_hint,
            #[allow(deprecated)]
            kind: Some(coordinate.kind),
        }
    }

    /// Create a new [`CommentTarget`] pointing to a specific external content.
    #[inline]
    pub fn external(content: &'a ExternalContentId, hint: Option<&'a Url>) -> Self {
        Self::External { content, hint }
    }

    /// Sets the relay hint for the event or coordinate.
    #[inline]
    pub fn relay_hint(self, relay_hint: &'a RelayUrl) -> Self {
        match self {
            Self::Event {
                id,
                pubkey_hint,
                kind,
                ..
            } => Self::Event {
                id,
                pubkey_hint,
                kind,
                relay_hint: Some(relay_hint),
            },
            #[allow(deprecated)]
            Self::Coordinate { address, kind, .. } => Self::Coordinate {
                address,
                kind,
                relay_hint: Some(relay_hint),
            },
            _ => self,
        }
    }

    /// Converts the comment target into a vector of tags
    ///
    /// ## Example
    ///
    /// If the target is `event` and `is_root` is true will return
    ///
    /// ```json
    /// [
    ///   ["E", "<event-id>", "<relay-hint>", "<public-key>"],
    ///   ["P", "<public-key>"],
    ///   ["K", "<event-kind>"]
    /// ]
    /// ```
    pub fn as_vec(&self, is_root: bool) -> Vec<Tag> {
        let mut tags = Vec::new();

        match self {
            Self::Event {
                id,
                relay_hint,
                pubkey_hint,
                kind,
            } => {
                tags.reserve_exact(
                    1 + usize::from(pubkey_hint.is_some()) + usize::from(kind.is_some()),
                );
                tags.push(Tag::from_standardized_without_cell(TagStandard::Event {
                    event_id: **id,
                    relay_url: relay_hint.cloned(),
                    marker: None,
                    public_key: pubkey_hint.copied(),
                    uppercase: is_root,
                }));

                if let Some(pubkey) = pubkey_hint {
                    tags.push(Tag::from_standardized_without_cell(
                        TagStandard::PublicKey {
                            public_key: **pubkey,
                            relay_url: relay_hint.cloned(),
                            alias: None,
                            uppercase: is_root,
                        },
                    ));
                }

                if let Some(kind) = kind {
                    tags.push(Tag::from_standardized_without_cell(TagStandard::Kind {
                        kind: **kind,
                        uppercase: is_root,
                    }));
                }
            }
            Self::Coordinate {
                address,
                relay_hint,
                ..
            } => {
                tags.reserve_exact(3);
                tags.push(Tag::from_standardized_without_cell(
                    TagStandard::Coordinate {
                        coordinate: address.into_owned(),
                        relay_url: relay_hint.cloned(),
                        uppercase: is_root,
                    },
                ));
                tags.push(Tag::from_standardized_without_cell(
                    TagStandard::PublicKey {
                        public_key: *address.public_key,
                        relay_url: relay_hint.cloned(),
                        alias: None,
                        uppercase: is_root,
                    },
                ));
                tags.push(Tag::from_standardized_without_cell(TagStandard::Kind {
                    kind: *address.kind,
                    uppercase: is_root,
                }));
            }
            Self::External { content, hint } => {
                tags.reserve_exact(2);
                tags.push(Tag::from_standardized_without_cell(
                    TagStandard::ExternalContent {
                        content: ExternalContentId::clone(content),
                        hint: hint.cloned(),
                        uppercase: is_root,
                    },
                ));
                tags.push(Tag::from_standardized_without_cell(
                    TagStandard::Nip73Kind {
                        kind: content.kind(),
                        uppercase: is_root,
                    },
                ))
            }
        }

        tags
    }
}

impl<'a> From<&'a Event> for CommentTarget<'a> {
    fn from(event: &'a Event) -> Self {
        if let Some(coordinate) = event.coordinate() {
            CommentTarget::coordinate(coordinate, None)
        } else {
            CommentTarget::event(&event.id, &event.kind, Some(&event.pubkey), None)
        }
    }
}

/// Extract NIP22 root target
pub fn extract_root(event: &Event) -> Option<CommentTarget> {
    extract_data(event, true)
}

/// Extract NIP22 parent target
pub fn extract_parent(event: &Event) -> Option<CommentTarget> {
    extract_data(event, false)
}

fn extract_data(event: &Event, is_root: bool) -> Option<CommentTarget> {
    if event.kind != Kind::Comment {
        return None;
    }

    // Try to extract event
    if let Some((event_id, relay_hint, public_key)) = extract_event(event, is_root) {
        return Some(CommentTarget::Event {
            id: event_id,
            relay_hint,
            pubkey_hint: public_key,
            kind: extract_kind(event, is_root),
        });
    }

    // Try to extract coordinate
    if let Some((address, relay_hint)) = extract_coordinate(event, is_root) {
        // Extract kind
        // TODO: for now we allow optional `k`/`K` tag, but according to NIP-22, it should be mandatory.
        let kind: Option<&Kind> = extract_kind(event, is_root);

        // Check if matches the address kind
        if let Some(kind) = kind {
            if kind != address.kind {
                return None;
            }
        }

        return Some(CommentTarget::Coordinate {
            address,
            relay_hint,
            #[allow(deprecated)]
            kind,
        });
    }

    if let Some((content, hint)) = extract_external(event, is_root) {
        return Some(CommentTarget::External { content, hint });
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
fn extract_coordinate(
    event: &Event,
    is_root: bool,
) -> Option<(CoordinateBorrow<'_>, Option<&RelayUrl>)> {
    event
        .tags
        .filter_standardized(TagKind::single_letter(Alphabet::A, is_root))
        .find_map(|tag| match tag {
            TagStandard::Coordinate {
                coordinate,
                relay_url,
                uppercase,
                ..
            } => check_return(
                (coordinate.borrow(), relay_url.as_ref()),
                is_root,
                *uppercase,
            ),
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

#[cfg(test)]
#[cfg(feature = "std")]
mod tests {
    use super::*;
    use crate::prelude::*;

    fn check_kind(tags: &[Tag], kind: Kind, uppercase: bool) {
        assert!(
            tags.contains(&Tag::from_standardized_without_cell(TagStandard::Kind {
                kind,
                uppercase
            }))
        );
    }

    fn check_nip73_kind(tags: &[Tag], kind: Nip73Kind, uppercase: bool) {
        assert!(tags.contains(&Tag::from_standardized_without_cell(
            TagStandard::Nip73Kind { kind, uppercase }
        )));
    }

    fn check_pubkey(tags: &[Tag], public_key: PublicKey, uppercase: bool) {
        assert!(tags.contains(&Tag::from_standardized_without_cell(
            TagStandard::PublicKey {
                public_key,
                relay_url: None,
                alias: None,
                uppercase
            }
        )));
    }

    #[test]
    fn test_event() {
        let keys = Keys::generate();
        let kind = Kind::GitPatch;
        let event_id = EventId::new(
            &keys.public_key(),
            &Timestamp::from_secs(1),
            &kind,
            &Tags::new(),
            "",
        );

        let comment_target = CommentTarget::event(&event_id, &kind, Some(&keys.public_key), None);

        // Root
        let root_vec = comment_target.as_vec(true);
        assert!(
            root_vec.contains(&Tag::from_standardized_without_cell(TagStandard::Event {
                event_id,
                relay_url: None,
                marker: None,
                public_key: Some(keys.public_key()),
                uppercase: true
            }))
        );
        check_pubkey(&root_vec, keys.public_key(), true);
        check_kind(&root_vec, kind, true);

        // Parent
        let parent_vec = comment_target.as_vec(false);
        assert!(
            parent_vec.contains(&Tag::from_standardized_without_cell(TagStandard::Event {
                event_id,
                relay_url: None,
                marker: None,
                public_key: Some(keys.public_key()),
                uppercase: false
            }))
        );
        check_pubkey(&parent_vec, keys.public_key(), false);
        check_kind(&parent_vec, kind, false);
    }

    #[test]
    fn test_coordinate() {
        let keys = Keys::generate();
        let kind = Kind::ContactList;
        let coordinate = Coordinate::new(kind, keys.public_key());

        let comment_target = CommentTarget::coordinate(coordinate.borrow(), None);

        // Root
        let root_vec = comment_target.as_vec(true);
        assert!(root_vec.contains(&Tag::from_standardized_without_cell(
            TagStandard::Coordinate {
                coordinate: coordinate.clone(),
                relay_url: None,
                uppercase: true
            }
        )));
        check_pubkey(&root_vec, keys.public_key(), true);
        check_kind(&root_vec, kind, true);

        // Parent
        let parent_vec = comment_target.as_vec(false);
        assert!(parent_vec.contains(&Tag::coordinate(coordinate, None)));
        check_pubkey(&parent_vec, keys.public_key(), false);
        check_kind(&parent_vec, kind, false);
    }

    #[test]
    fn test_external_content() {
        let external_content = ExternalContentId::Url("https://rust-nostr.org".parse().unwrap());
        let kind = external_content.kind();

        let comment_target = CommentTarget::external(&external_content, None);

        // Root
        let root_vec = comment_target.as_vec(true);
        assert!(root_vec.contains(&Tag::from_standardized_without_cell(
            TagStandard::ExternalContent {
                content: external_content.clone(),
                hint: None,
                uppercase: true
            }
        )));
        check_nip73_kind(&root_vec, kind.clone(), true);

        // Parent
        let parent_vec = comment_target.as_vec(false);
        assert!(parent_vec.contains(&Tag::from_standardized_without_cell(
            TagStandard::ExternalContent {
                content: external_content.clone(),
                hint: None,
                uppercase: false
            }
        )));
        check_nip73_kind(&parent_vec, kind, false);
    }
}
