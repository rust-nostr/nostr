// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use core::ops::Deref;
use core::str::FromStr;

use nostr::hashes::sha1::Hash as Sha1Hash;
use nostr_sdk::prelude::*;
use wasm_bindgen::prelude::*;

use crate::protocol::key::JsPublicKey;
use crate::protocol::nips::nip01::JsCoordinate;

/// Git Repository Announcement
///
/// Git repositories are hosted in Git-enabled servers, but their existence can be announced using Nostr events,
/// as well as their willingness to receive patches, bug reports and comments in general.
#[wasm_bindgen(js_name = GitRepositoryAnnouncement)]
pub struct JsGitRepositoryAnnouncement {
    /// Repository ID (usually kebab-case short name)
    #[wasm_bindgen(getter_with_clone)]
    pub id: String,
    /// Human-readable project name
    #[wasm_bindgen(getter_with_clone)]
    pub name: Option<String>,
    /// Brief human-readable project description
    #[wasm_bindgen(getter_with_clone)]
    pub description: Option<String>,
    /// Webpage urls, if the git server being used provides such a thing
    #[wasm_bindgen(getter_with_clone)]
    pub web: Vec<String>,
    /// Urls for git-cloning
    #[wasm_bindgen(getter_with_clone)]
    pub clone: Vec<String>,
    /// Relays that this repository will monitor for patches and issues
    #[wasm_bindgen(getter_with_clone)]
    pub relays: Vec<String>,
    /// Earliest unique commit ID
    ///
    /// `euc` marker should be the commit ID of the earliest unique commit of this repo,
    /// made to identify it among forks and group it with other repositories hosted elsewhere that may represent essentially the same project.
    /// In most cases it will be the root commit of a repository.
    /// In case of a permanent fork between two projects, then the first commit after the fork should be used.
    #[wasm_bindgen(getter_with_clone)]
    pub euc: Option<String>,
    /// Other recognized maintainers
    #[wasm_bindgen(getter_with_clone)]
    pub maintainers: Vec<JsPublicKey>,
}

impl From<JsGitRepositoryAnnouncement> for GitRepositoryAnnouncement {
    fn from(value: JsGitRepositoryAnnouncement) -> Self {
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
            maintainers: value.maintainers.into_iter().map(|p| *p).collect(),
        }
    }
}

/// Git Issue
#[wasm_bindgen(js_name = GitIssue)]
pub struct JsGitIssue {
    /// The repository address
    #[wasm_bindgen(getter_with_clone)]
    pub repository: JsCoordinate,
    /// The issue content (markdown)
    #[wasm_bindgen(getter_with_clone)]
    pub content: String,
    /// Subject
    #[wasm_bindgen(getter_with_clone)]
    pub subject: Option<String>,
    /// Labels
    #[wasm_bindgen(getter_with_clone)]
    pub labels: Vec<String>,
}

impl From<JsGitIssue> for GitIssue {
    fn from(value: JsGitIssue) -> Self {
        Self {
            repository: value.repository.deref().clone(),
            content: value.content,
            subject: value.subject,
            labels: value.labels,
        }
    }
}
