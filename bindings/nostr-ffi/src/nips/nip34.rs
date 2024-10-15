// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;
use std::sync::Arc;

use nostr::nips::nip34;
use nostr::Url;
use uniffi::Record;

use crate::nips::nip01::Coordinate;
use crate::PublicKey;

/// Git Repository Announcement
///
/// Git repositories are hosted in Git-enabled servers, but their existence can be announced using Nostr events,
/// as well as their willingness to receive patches, bug reports and comments in general.
#[derive(Record)]
pub struct GitRepositoryAnnouncement {
    /// Repository ID (usually kebab-case short name)
    pub id: String,
    /// Human-readable project name
    pub name: Option<String>,
    /// Brief human-readable project description
    pub description: Option<String>,
    /// Webpage urls, if the git server being used provides such a thing
    pub web: Vec<String>,
    /// Urls for git-cloning
    pub clone: Vec<String>,
    /// Relays that this repository will monitor for patches and issues
    pub relays: Vec<String>,
    /// Earliest unique commit ID
    ///
    /// `euc` marker should be the commit ID of the earliest unique commit of this repo,
    /// made to identify it among forks and group it with other repositories hosted elsewhere that may represent essentially the same project.
    /// In most cases it will be the root commit of a repository.
    /// In case of a permanent fork between two projects, then the first commit after the fork should be used.
    pub euc: Option<String>,
    /// Other recognized maintainers
    pub maintainers: Vec<Arc<PublicKey>>,
}

impl From<GitRepositoryAnnouncement> for nip34::GitRepositoryAnnouncement {
    fn from(value: GitRepositoryAnnouncement) -> Self {
        Self {
            id: value.id,
            name: value.name,
            description: value.description,
            web: value
                .web
                .into_iter()
                .filter_map(|u| Url::parse(&u).ok())
                .collect(),
            clone: value
                .clone
                .into_iter()
                .filter_map(|u| Url::parse(&u).ok())
                .collect(),
            relays: value
                .relays
                .into_iter()
                .filter_map(|u| Url::parse(&u).ok())
                .collect(),
            euc: value.euc,
            maintainers: value.maintainers.into_iter().map(|p| **p).collect(),
        }
    }
}

/// Git Issue
#[derive(Record)]
pub struct GitIssue {
    /// The issue content (markdown)
    pub content: String,
    /// The repository address
    pub repository: Arc<Coordinate>,
    /// Public keys (owners or other users)
    pub public_keys: Vec<Arc<PublicKey>>,
    /// Subject
    pub subject: Option<String>,
    /// Labels
    pub labels: Vec<String>,
}

impl From<GitIssue> for nip34::GitIssue {
    fn from(value: GitIssue) -> Self {
        Self {
            content: value.content,
            repository: value.repository.as_ref().deref().clone(),
            public_keys: value.public_keys.into_iter().map(|p| **p).collect(),
            subject: value.subject,
            labels: value.labels,
        }
    }
}
