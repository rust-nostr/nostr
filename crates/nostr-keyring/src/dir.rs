// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use thiserror::Error;

pub(crate) const ACCOUNT_EXTENSION: &str = "ncryptsec";
pub(crate) const ACCOUNT_DOT_EXTENSION: &str = ".ncryptsec";

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error("Impossible to get file name")]
    FailedToGetFileName,
}

pub fn accounts_dir<P>(base_path: P) -> Result<PathBuf, Error>
where
    P: AsRef<Path>,
{
    let base_path: &Path = base_path.as_ref();
    fs::create_dir_all(&base_path)?;

    let accounts_path: PathBuf = base_path.join("keys");
    Ok(accounts_path)
}

pub fn get_accounts_list<P>(path: P) -> Result<BTreeSet<String>, Error>
where
    P: AsRef<Path>,
{
    let mut names: BTreeSet<String> = BTreeSet::new();

    // Get and iterate all paths
    let paths = fs::read_dir(path)?;
    for path in paths {
        let path: PathBuf = path?.path();

        // Check if path has file name
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            // Check if file name terminate with extension
            if name.ends_with(ACCOUNT_DOT_EXTENSION) {
                // Split file name and extension
                let mut split = name.split(ACCOUNT_DOT_EXTENSION);
                if let Some(value) = split.next() {
                    names.insert(value.to_string());
                }
            }
        }
    }

    Ok(names)
}

pub(crate) fn get_account_file<P, S>(base_path: P, name: S) -> Result<PathBuf, Error>
where
    P: AsRef<Path>,
    S: Into<String>,
{
    let mut keychain_file: PathBuf = base_path.as_ref().join(name.into());
    keychain_file.set_extension(ACCOUNT_EXTENSION);
    Ok(keychain_file)
}
