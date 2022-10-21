// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("error: {msg}")]
    Generic { msg: String },
}

impl From<anyhow::Error> for Error {
    fn from(e: anyhow::Error) -> Error {
        Self::Generic { msg: e.to_string() }
    }
}
