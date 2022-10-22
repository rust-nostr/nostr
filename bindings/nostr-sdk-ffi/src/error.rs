// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

#[derive(thiserror::Error, Debug)]
pub enum NostrError {
    #[error("error: {msg}")]
    Generic { msg: String },
}

impl From<anyhow::Error> for NostrError {
    fn from(e: anyhow::Error) -> NostrError {
        Self::Generic { msg: e.to_string() }
    }
}
