// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP34: git stuff
//!
//! <https://github.com/nostr-protocol/nips/blob/master/34.md>

#![allow(clippy::wrong_self_convention)]

use alloc::borrow::Cow;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt;

use hashes::sha1::Hash as Sha1Hash;

use crate::event::builder::{Error, EventBuilder, WrongKindError};
use crate::nips::nip01::{self, Coordinate};
use crate::nips::nip10::Marker;
use crate::types::url::Url;
use crate::{EventId, Kind, PublicKey, RelayUrl, Tag, TagKind, TagStandard, Timestamp};

/// Earlier unique commit ID marker
pub const EUC: &str = "euc";

const GIT_PATCH_ALT: &str = "git patch";
const GIT_PATCH_COVER_LETTER_ALT: &str = "git patch cover letter";

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
    pub relays: Vec<RelayUrl>,
    /// Earliest unique commit ID
    ///
    /// `euc` marker should be the commit ID of the earliest unique commit of this repo,
    /// made to identify it among forks and group it with other repositories hosted elsewhere that may represent essentially the same project.
    /// In most cases it will be the root commit of a repository.
    /// In case of a permanent fork between two projects, then the first commit after the fork should be used.
    pub euc: Option<Sha1Hash>,
    /// Other recognized maintainers
    pub maintainers: Vec<PublicKey>,
}

impl GitRepositoryAnnouncement {
    pub(crate) fn to_event_builder(self) -> Result<EventBuilder, Error> {
        if self.id.is_empty() {
            // TODO: should return another error?
            return Err(Error::NIP01(nip01::Error::InvalidCoordinate));
        }

        let mut tags: Vec<Tag> = Vec::with_capacity(1);

        // Add repo ID
        tags.push(Tag::identifier(self.id));

        // Add name
        if let Some(name) = self.name {
            tags.push(Tag::from_standardized_without_cell(TagStandard::Name(name)));
        }

        // Add description
        if let Some(description) = self.description {
            tags.push(Tag::from_standardized_without_cell(
                TagStandard::Description(description),
            ));
        }

        // Add web
        if !self.web.is_empty() {
            tags.push(Tag::from_standardized_without_cell(TagStandard::Web(
                self.web,
            )));
        }

        // Add clone
        if !self.clone.is_empty() {
            tags.push(Tag::from_standardized_without_cell(TagStandard::GitClone(
                self.clone,
            )));
        }

        // Add relays
        if !self.relays.is_empty() {
            tags.push(Tag::from_standardized_without_cell(TagStandard::Relays(
                self.relays,
            )));
        }

        // Add EUC
        if let Some(commit) = self.euc {
            tags.push(Tag::from_standardized_without_cell(
                TagStandard::GitEarliestUniqueCommitId(commit),
            ));
        }

        // Add maintainers
        if !self.maintainers.is_empty() {
            tags.push(Tag::from_standardized_without_cell(
                TagStandard::GitMaintainers(self.maintainers),
            ));
        }

        // Build
        Ok(EventBuilder::new(Kind::GitRepoAnnouncement, "").tags(tags))
    }
}

/// Git Issue
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GitIssue {
    /// The repository address
    pub repository: Coordinate,
    /// The issue content (markdown)
    pub content: String,
    /// Subject
    pub subject: Option<String>,
    /// Labels
    pub labels: Vec<String>,
}

impl GitIssue {
    /// Based on <https://github.com/nostr-protocol/nips/blob/ea36ec9ed7596e49bf7f217b05954c1fecacad88/34.md> revision.
    pub(crate) fn to_event_builder(self) -> Result<EventBuilder, Error> {
        // Check if repository address kind is wrong
        if self.repository.kind != Kind::GitRepoAnnouncement {
            return Err(Error::WrongKind {
                received: self.repository.kind,
                expected: WrongKindError::Single(Kind::GitRepoAnnouncement),
            });
        }

        // Verify coordinate
        self.repository.verify()?;

        let owner_public_key: PublicKey = self.repository.public_key;

        let mut tags: Vec<Tag> = Vec::with_capacity(2);

        // Add coordinate
        tags.push(Tag::coordinate(self.repository));

        // Add owner public key
        tags.push(Tag::public_key(owner_public_key));

        // Add subject
        if let Some(subject) = self.subject {
            tags.push(Tag::from_standardized_without_cell(TagStandard::Subject(
                subject,
            )));
        }

        // Add labels
        tags.extend(self.labels.into_iter().map(Tag::hashtag));

        // Build
        Ok(EventBuilder::new(Kind::GitIssue, self.content).tags(tags))
    }
}

/// Git Patch Committer
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GitPatchCommitter {
    /// Name
    pub name: Option<String>,
    /// Email
    pub email: Option<String>,
    /// Timestamp
    pub timestamp: Timestamp,
    /// Timezone offset in minutes
    pub offset_minutes: i32,
}

/// Git Patch Content
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GitPatchContent {
    /// Cover letter
    CoverLetter {
        /// Title
        title: String,
        /// Description
        description: String,
        /// Last commit
        last_commit: Sha1Hash,
        /// Number of commits
        commits_len: usize,
    },
    /// Patch
    Patch {
        /// Patch content
        content: String,
        /// Commit hash
        commit: Sha1Hash,
        /// Parent commit
        parent_commit: Sha1Hash,
        /// PGP signature of commit
        commit_pgp_sig: Option<String>,
        /// Committer
        committer: GitPatchCommitter,
    },
}

impl fmt::Display for GitPatchContent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CoverLetter {
                title,
                description,
                last_commit,
                commits_len,
            } => {
                write!(f, "From {last_commit} Mon Sep 17 00:00:00 2001\nSubject: [PATCH 0/{commits_len}] {title}\n\n{description}")
            }
            Self::Patch { content, .. } => write!(f, "{content}"),
        }
    }
}

/// Git Patch
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GitPatch {
    /// Repository ID
    pub repo_id: String,
    /// Patch
    pub content: GitPatchContent,
    /// Maintainers
    pub maintainers: Vec<PublicKey>,
    /// Earliest unique commit ID of repo
    pub euc: String,
    /// Root proposal ID
    pub root_proposal_id: Option<EventId>,
}

impl GitPatch {
    pub(crate) fn to_event_builder(self) -> EventBuilder {
        let content: String = self.content.to_string();

        let mut tags: Vec<Tag> = Vec::with_capacity(2);

        // Add coordinate
        tags.reserve_exact(self.maintainers.len());
        tags.extend(self.maintainers.iter().copied().map(|p| {
            Tag::coordinate(
                Coordinate::new(Kind::GitRepoAnnouncement, p).identifier(self.repo_id.clone()),
            )
        }));

        // Add EUC (as reference, not with `euc` marker)
        tags.push(Tag::reference(self.euc));

        // Handle patch content
        match self.content {
            GitPatchContent::CoverLetter { title, .. } => {
                // Add cover letter tags
                tags.reserve_exact(2);
                tags.push(Tag::hashtag("cover-letter"));
                tags.push(Tag::alt(format!("{GIT_PATCH_COVER_LETTER_ALT}: {title}")));
            }
            GitPatchContent::Patch {
                commit,
                parent_commit,
                commit_pgp_sig,
                committer,
                ..
            } => {
                tags.reserve_exact(6);
                tags.push(Tag::reference(commit.to_string()));
                tags.push(Tag::from_standardized_without_cell(TagStandard::GitCommit(
                    commit,
                )));
                tags.push(Tag::custom(
                    TagKind::Custom(Cow::Borrowed("parent-commit")),
                    vec![parent_commit.to_string()],
                ));
                tags.push(Tag::custom(
                    TagKind::Custom(Cow::Borrowed("commit-pgp-sig")),
                    vec![commit_pgp_sig.unwrap_or_default()],
                ));
                tags.push(Tag::custom(
                    TagKind::Custom(Cow::Borrowed("committer")),
                    vec![
                        committer.name.unwrap_or_default(),
                        committer.email.unwrap_or_default(),
                        committer.timestamp.to_string(),
                        committer.offset_minutes.to_string(),
                    ],
                ));
                tags.push(Tag::alt(GIT_PATCH_ALT));
            }
        }

        // Handle root proposal ID
        match self.root_proposal_id {
            Some(root_proposal_id) => {
                tags.reserve_exact(3);
                tags.push(Tag::hashtag("root"));
                tags.push(Tag::hashtag("revision-root"));
                tags.push(Tag::from_standardized_without_cell(TagStandard::Event {
                    event_id: root_proposal_id,
                    relay_url: None,
                    marker: Some(Marker::Reply),
                    public_key: None,
                    uppercase: false,
                }));
            }
            None => tags.push(Tag::hashtag("root")),
        }

        // Add public keys
        tags.reserve_exact(self.maintainers.len());
        tags.extend(self.maintainers.into_iter().map(Tag::public_key));

        // Build
        EventBuilder::new(Kind::GitPatch, content).tags(tags)
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use core::str::FromStr;

    use super::*;
    use crate::{Event, Keys, Tags};

    #[test]
    fn test_git_repo_announcement() {
        let repo = GitRepositoryAnnouncement {
            id: String::from("test"),
            name: Some(String::from("Test nostr repository")),
            description: Some(String::from("Long desc")),
            web: Vec::new(),
            clone: vec![Url::parse("https://github.com/rust-nostr/nostr.git").unwrap()],
            relays: vec![
                RelayUrl::parse("wss://example.com").unwrap(),
                RelayUrl::parse("wss://example.org").unwrap(),
            ],
            euc: Some(Sha1Hash::from_str("aa231c4c6a5777dc89b42207b499891a344add5c").unwrap()),
            maintainers: vec![PublicKey::parse(
                "npub1drvpzev3syqt0kjrls50050uzf25gehpz9vgdw08hvex7e0vgfeq0eseet",
            )
            .unwrap()],
        };

        let keys = Keys::generate();
        let event: Event = repo
            .to_event_builder()
            .unwrap()
            .sign_with_keys(&keys)
            .unwrap();

        assert_eq!(event.kind, Kind::GitRepoAnnouncement);
        assert!(event.content.is_empty());

        let tags = Tags::parse([
            vec!["d", "test"],
            vec!["name", "Test nostr repository"],
            vec!["description", "Long desc"],
            vec!["clone", "https://github.com/rust-nostr/nostr.git"],
            vec!["relays", "wss://example.com", "wss://example.org"],
            vec!["r", "aa231c4c6a5777dc89b42207b499891a344add5c", "euc"],
            vec![
                "maintainers",
                "68d81165918100b7da43fc28f7d1fc12554466e1115886b9e7bb326f65ec4272",
            ],
        ])
        .unwrap();
        assert_eq!(event.tags, tags);
    }

    #[test]
    fn test_git_issue() {
        let pk =
            PublicKey::parse("npub1drvpzev3syqt0kjrls50050uzf25gehpz9vgdw08hvex7e0vgfeq0eseet")
                .unwrap();
        let repository = Coordinate::new(Kind::GitRepoAnnouncement, pk).identifier("rust-nostr");

        let repo = GitIssue {
            repository,
            content: String::from("My issue content"),
            subject: Some(String::from("My issue subject")),
            labels: vec![String::from("bug")],
        };

        let keys = Keys::generate();
        let event: Event = repo
            .to_event_builder()
            .unwrap()
            .sign_with_keys(&keys)
            .unwrap();

        assert_eq!(event.kind, Kind::GitIssue);
        assert_eq!(event.content, "My issue content");

        let tags = Tags::parse([
            vec![
                "a",
                "30617:68d81165918100b7da43fc28f7d1fc12554466e1115886b9e7bb326f65ec4272:rust-nostr",
            ],
            vec![
                "p",
                "68d81165918100b7da43fc28f7d1fc12554466e1115886b9e7bb326f65ec4272",
            ],
            vec!["subject", "My issue subject"],
            vec!["t", "bug"],
        ])
        .unwrap();
        assert_eq!(event.tags, tags);
    }
}
