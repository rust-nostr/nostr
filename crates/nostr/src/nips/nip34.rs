// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP34: git stuff
//!
//! <https://github.com/nostr-protocol/nips/blob/master/34.md>

#![allow(clippy::wrong_self_convention)]

use alloc::string::String;
use alloc::vec::Vec;

use crate::nips::nip01::Coordinate;
use crate::types::url::Url;
use crate::{EventBuilder, Kind, PublicKey, Tag, TagStandard};

/// Earlier unique commit ID
pub const EUC: &str = "euc";

/// Git Repository Announcement
///
/// Git repositories are hosted in Git-enabled servers, but their existence can be announced using Nostr events,
/// as well as their willingness to receive patches, bug reports and comments in general.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GitRepositoryAnnouncement {
    /// Repository ID (usually kebab-case short name)
    pub id: String,
    /// Human-readable project name
    pub name: Option<String>,
    /// Brief human-readable project description
    pub description: Option<String>,
    /// Webpage urls, if the git server being used provides such a thing
    pub web: Vec<Url>,
    /// Urls for git-cloning
    pub clone: Vec<Url>,
    /// Relays that this repository will monitor for patches and issues
    pub relays: Vec<Url>,
    /// Earliest unique commit ID
    ///
    /// `euc` marker should be the commit ID of the earliest unique commit of this repo,
    /// made to identify it among forks and group it with other repositories hosted elsewhere that may represent essentially the same project.
    /// In most cases it will be the root commit of a repository.
    /// In case of a permanent fork between two projects, then the first commit after the fork should be used.
    pub euc: Option<String>,
    /// Other recognized maintainers
    pub maintainers: Vec<PublicKey>,
}

impl GitRepositoryAnnouncement {
    pub(crate) fn to_event_builder(self) -> EventBuilder {
        let tags: Vec<Tag> = self.into();
        EventBuilder::new(Kind::GitRepoAnnouncement, "", tags)
    }
}

impl From<GitRepositoryAnnouncement> for Vec<Tag> {
    fn from(value: GitRepositoryAnnouncement) -> Self {
        let mut tags: Self = Self::with_capacity(1);

        // Add repo ID
        tags.push(Tag::identifier(value.id));

        // Add name
        if let Some(name) = value.name {
            tags.push(Tag::from_standardized_without_cell(TagStandard::Name(name)));
        }

        // Add description
        if let Some(description) = value.description {
            tags.push(Tag::from_standardized_without_cell(
                TagStandard::Description(description),
            ));
        }

        // Add web
        if !value.web.is_empty() {
            tags.push(Tag::from_standardized_without_cell(TagStandard::Web(
                value.web,
            )));
        }

        // Add clone
        if !value.clone.is_empty() {
            tags.push(Tag::from_standardized_without_cell(TagStandard::GitClone(
                value.clone,
            )));
        }

        // Add relays
        if !value.relays.is_empty() {
            tags.push(Tag::from_standardized_without_cell(TagStandard::Relays(
                value.relays,
            )));
        }

        // Add EUC
        if let Some(euc) = value.euc {
            tags.push(Tag::from_standardized_without_cell(
                TagStandard::GitEarliestUniqueCommitId(euc),
            ));
        }

        if !value.maintainers.is_empty() {
            tags.push(Tag::from_standardized_without_cell(
                TagStandard::GitMaintainers(value.maintainers),
            ));
        }

        tags
    }
}

/// Git Issue
pub struct GitIssue {
    /// The issue content (markdown)
    pub content: String,
    /// The repository address
    pub repository: Coordinate,
    /// Public keys (owners or other users)
    pub public_keys: Vec<PublicKey>,
    /// Subject
    pub subject: Option<String>,
    /// Labels
    pub labels: Vec<String>,
}

impl GitIssue {
    pub(crate) fn to_event_builder(self) -> EventBuilder {
        let mut tags: Vec<Tag> = Vec::with_capacity(1);

        // Add coordinate
        tags.push(Tag::coordinate(self.repository));

        // Add public keys
        tags.extend(self.public_keys.into_iter().map(Tag::public_key));

        // Add subject
        if let Some(subject) = self.subject {
            tags.push(Tag::from_standardized_without_cell(TagStandard::Subject(
                subject,
            )));
        }

        // Add labels
        tags.extend(self.labels.into_iter().map(Tag::hashtag));

        EventBuilder::new(Kind::GitIssue, self.content, tags)
    }
}
