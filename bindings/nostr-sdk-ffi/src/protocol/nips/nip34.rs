// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;
use std::str::FromStr;
use std::sync::Arc;

use nostr::hashes::sha1::Hash as Sha1Hash;
use nostr::nips::nip34;
use nostr::{RelayUrl, Url};
use uniffi::{Enum, Record};

use crate::error::NostrSdkError;
use crate::protocol::key::PublicKey;
use crate::protocol::nips::nip01::Coordinate;
use crate::protocol::types::Timestamp;

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
                .filter_map(|u| RelayUrl::parse(&u).ok())
                .collect(),
            euc: value.euc.and_then(|euc| Sha1Hash::from_str(&euc).ok()),
            maintainers: value.maintainers.into_iter().map(|p| **p).collect(),
        }
    }
}

/// Git Issue
#[derive(Record)]
pub struct GitIssue {
    /// The repository address
    pub repository: Arc<Coordinate>,
    /// The issue content (markdown)
    pub content: String,
    /// Subject
    pub subject: Option<String>,
    /// Labels
    pub labels: Vec<String>,
}

impl From<GitIssue> for nip34::GitIssue {
    fn from(value: GitIssue) -> Self {
        Self {
            repository: value.repository.as_ref().deref().clone(),
            content: value.content,
            subject: value.subject,
            labels: value.labels,
        }
    }
}

/// Git Patch Committer
#[derive(Record)]
pub struct GitPatchCommitter {
    /// Name
    pub name: Option<String>,
    /// Email
    pub email: Option<String>,
    /// Timestamp
    pub timestamp: Arc<Timestamp>,
    /// Timezone offset in minutes
    pub offset_minutes: i32,
}

impl From<GitPatchCommitter> for nip34::GitPatchCommitter {
    fn from(value: GitPatchCommitter) -> Self {
        Self {
            name: value.name,
            email: value.email,
            timestamp: **value.timestamp,
            offset_minutes: value.offset_minutes,
        }
    }
}

/// Git Patch Content
#[derive(Enum)]
pub enum GitPatchContent {
    /// Cover letter
    CoverLetter {
        /// Title
        title: String,
        /// Description
        description: String,
        /// Last commit
        last_commit: String,
        /// Number of commits
        commits_len: u64,
    },
    /// Patch
    Patch {
        /// Patch content
        content: String,
        /// Commit hash
        commit: String,
        /// Parent commit
        parent_commit: String,
        /// PGP signature of commit
        commit_pgp_sig: Option<String>,
        /// Committer
        committer: GitPatchCommitter,
    },
}

impl TryFrom<GitPatchContent> for nip34::GitPatchContent {
    type Error = NostrSdkError;

    fn try_from(value: GitPatchContent) -> Result<Self, Self::Error> {
        match value {
            GitPatchContent::CoverLetter {
                title,
                description,
                last_commit,
                commits_len,
            } => Ok(Self::CoverLetter {
                title,
                description,
                last_commit: Sha1Hash::from_str(&last_commit)?,
                commits_len: commits_len as usize,
            }),
            GitPatchContent::Patch {
                content,
                commit,
                parent_commit,
                commit_pgp_sig,
                committer,
            } => Ok(Self::Patch {
                content,
                commit: Sha1Hash::from_str(&commit)?,
                parent_commit: Sha1Hash::from_str(&parent_commit)?,
                commit_pgp_sig,
                committer: committer.into(),
            }),
        }
    }
}

/// Git Patch
#[derive(Record)]
pub struct GitPatch {
    /// Repository
    pub repository: Arc<Coordinate>,
    /// Patch
    pub content: GitPatchContent,
    /// Earliest unique commit ID of repo
    pub euc: String,
    /// Labels
    pub labels: Vec<String>,
}

impl TryFrom<GitPatch> for nip34::GitPatch {
    type Error = NostrSdkError;

    fn try_from(value: GitPatch) -> Result<Self, Self::Error> {
        Ok(Self {
            repository: value.repository.as_ref().deref().clone(),
            content: value.content.try_into()?,
            euc: Sha1Hash::from_str(&value.euc)?,
            labels: value.labels,
        })
    }
}
