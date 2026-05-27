// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Blossom error

use nostr::types::url;
use reqwest::{Response, header};

opaquerr::define_kind! {
    /// Nostr blossom error kind.
    pub ErrorKind {
        /// Nostr protocol error.
        Protocol => "nostr protocol error",
        /// HTTP error
        Http => "HTTP error",
        /// Input is not well-formed and cannot be parsed.
        Malformed => "input is malformed",
        /// Input is well-formed, but violates a protocol/library invariant.
        Invalid => "input violates a protocol/library invariant",
        /// Anything not covered by the stable categories above.
        Other => "other error",
    }
}

opaquerr::define_error! {
    /// Nostr blossom error.
    pub Error(ErrorKind)

    from {
        nostr::error::Error => ErrorKind::Protocol,
        reqwest::Error => ErrorKind::Http,
        header::InvalidHeaderValue => ErrorKind::Http,
        url::ParseError => ErrorKind::Malformed,
        header::ToStrError => ErrorKind::Invalid,
    }
}

impl Error {
    pub(crate) fn response(prefix: &'static str, res: Response) -> Self {
        let reason: &str = res
            .headers()
            .get("X-Reason")
            .map(|h| h.to_str().unwrap_or("Unknown reason"))
            .unwrap_or_else(|| "No reason provided");
        let msg: String = format!("{prefix}: {} - {reason}", res.status());
        Self::new(ErrorKind::Http, msg)
    }
}
