// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("Pool error: {0}")]
    Pool(#[from] r2d2::Error),
    #[error(transparent)]
    Secp256k1(#[from] nostr::secp256k1::Error),
    #[error(transparent)]
    Hex(#[from] nostr::hashes::hex::Error),
    #[error(transparent)]
    Metadata(#[from] nostr::metadata::Error),
    #[error("Impossible to deserialize")]
    FailedToDeserialize,
    #[error("Impossible to serialize")]
    FailedToSerialize,
    #[error("Value not found")]
    ValueNotFound,
}
