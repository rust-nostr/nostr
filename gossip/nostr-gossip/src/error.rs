// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Nostr Gossip error

opaquerr::define_kind! {
    /// Nostr gossip error kind.
    pub ErrorKind {
        /// I/O error.
        IO => "I/O error",
        /// Storage error
        Storage => "storage error",
        /// Database migration error.
        Migration => "migration error",
        /// The operation is known but not supported.
        Unsupported => "the operation is known but not supported",
        /// Anything not covered by the stable categories above.
        Other => "other error",
    }
}

opaquerr::define_error! {
    /// Nostr gossip error.
    pub Error(ErrorKind)

    from {
        std::io::Error => ErrorKind::IO,
    }
}

impl Error {
    /// Storage error
    pub fn storage<E>(error: E) -> Self
    where
        E: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        Self::new(ErrorKind::Storage, error)
    }

    /// Migration error
    pub fn migration<E>(error: E) -> Self
    where
        E: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        Self::new(ErrorKind::Migration, error)
    }

    /// unsupported feature
    pub const fn unsupported(message: &'static str) -> Self {
        Self::with_static_message(ErrorKind::Unsupported, message)
    }
}
