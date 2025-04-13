// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! NIPC0: Code Snippets
//!
//! <https://github.com/nostr-protocol/nips/blob/master/C0.md>

use alloc::string::String;
use alloc::vec::Vec;

use crate::{EventBuilder, Kind, Tag, TagStandard};

/// Code snippet
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CodeSnippet {
    /// The code snippet.
    pub snippet: String,
    /// Programming language name.
    /// Examples: "javascript", "python", "rust"
    pub language: Option<String>,
    /// Name of the code snippet, commonly a filename.
    /// Examples: "hello-world.js", "quick-sort.py"
    pub name: Option<String>,
    /// File extension (without the dot).
    /// Examples: "js", "py", "rs"
    pub extension: Option<String>,
    /// Brief description of what the code does
    pub description: Option<String>,
    /// Runtime or environment specification.
    /// Example: "node v18.15.0", "python 3.11"
    pub runtime: Option<String>,
    /// License under which the code is shared.
    /// Example: "MIT", "GPL-3.0", "Apache-2.0"
    pub license: Option<String>,
    /// Dependencies required for the code to run.
    pub dependencies: Vec<String>,
    /// Reference to a repository where this code originates.
    pub repo: Option<String>,
}

impl CodeSnippet {
    /// Create a new code snippet
    #[inline]
    pub fn new<T>(snippet: T) -> Self
    where
        T: Into<String>,
    {
        Self {
            snippet: snippet.into(),
            ..Default::default()
        }
    }

    /// Set the programming language name (e.g. "javascript", "python", "rust").
    #[inline]
    pub fn language<T>(mut self, lang: T) -> Self
    where
        T: AsRef<str>,
    {
        self.language = Some(lang.as_ref().to_lowercase());
        self
    }

    /// Set the name of the code snippet, commonly a filename.
    #[inline]
    pub fn name<T>(mut self, name: T) -> Self
    where
        T: Into<String>,
    {
        self.name = Some(name.into());
        self
    }

    /// Set the file extension (without the dot).
    #[inline]
    pub fn extension<T>(mut self, extension: T) -> Self
    where
        T: Into<String>,
    {
        self.extension = Some(extension.into());
        self
    }

    /// Set a brief description of what the code does
    #[inline]
    pub fn description<T>(mut self, description: T) -> Self
    where
        T: Into<String>,
    {
        self.description = Some(description.into());
        self
    }

    /// Set the runtime or environment specification (e.g. "node v18.15.0", "python 3.11").
    #[inline]
    pub fn runtime<T>(mut self, runtime: T) -> Self
    where
        T: Into<String>,
    {
        self.runtime = Some(runtime.into());
        self
    }

    /// Set the license under which the code is shared (e.g. "MIT", "GPL-3.0", "Apache-2.0").
    #[inline]
    pub fn license<T>(mut self, license: T) -> Self
    where
        T: Into<String>,
    {
        self.license = Some(license.into());
        self
    }

    /// Add a dependency required for the code to run.
    pub fn dependencies<T>(mut self, dep: T) -> Self
    where
        T: Into<String>,
    {
        let dep = dep.into();
        if !self.dependencies.contains(&dep) {
            self.dependencies.push(dep);
        }
        self
    }

    /// Set the repository where this code originates.
    #[inline]
    pub fn repo<T>(mut self, repo: T) -> Self
    where
        T: Into<String>,
    {
        self.repo = Some(repo.into());
        self
    }

    /// Convert the code snippet to an event builder
    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn to_event_builder(self) -> EventBuilder {
        let mut tags: Vec<Tag> = Vec::new();

        let mut add_if_some = |tag: Option<TagStandard>| {
            if let Some(tag) = tag {
                tags.push(Tag::from_standardized_without_cell(tag));
            }
        };

        // `l` tag used for label in all event kinds except Code Snippets (1337)
        // is used as the programming language
        add_if_some(self.language.map(|l| TagStandard::Label {
            value: l,
            namespace: None,
        }));
        add_if_some(self.name.map(TagStandard::Name));
        add_if_some(self.extension.map(TagStandard::Extension));
        add_if_some(self.description.map(TagStandard::Description));
        add_if_some(self.runtime.map(TagStandard::Runtime));
        add_if_some(self.license.map(TagStandard::License));
        add_if_some(self.repo.map(TagStandard::Repository));

        for dep in self.dependencies.into_iter() {
            tags.push(TagStandard::Dependency(dep).into());
        }

        EventBuilder::new(Kind::CodeSnippet, self.snippet).tags(tags)
    }
}
