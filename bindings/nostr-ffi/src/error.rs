// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::fmt;

#[derive(Debug)]
pub enum NostrError {
    Generic { err: String },
}

impl fmt::Display for NostrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Generic { err } => write!(f, "{}", err),
        }
    }
}

impl From<anyhow::Error> for NostrError {
    fn from(e: anyhow::Error) -> NostrError {
        Self::Generic { err: e.to_string() }
    }
}
