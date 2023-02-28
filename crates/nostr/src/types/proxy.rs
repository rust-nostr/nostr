// Copyright (c) 2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Proxy

use serde_json::Value;

/// Proxy request
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ProxyRequest {
    /// NIP-05 proxy request
    Nip05(String),
    /// Unknown proxy request type
    Unknown {
        /// The type of data being proxied
        proxy_type: String,
        /// The request being proxied
        proxy_request: String,
    },
}

impl ProxyRequest {
    /// Create new `NIP-05` proxy request
    pub fn new_nip05(address: String) -> Self {
        Self::Nip05(address)
    }

    /// Create new `Unknown` proxy request
    pub fn new_unknown(proxy_type: String, proxy_request: String) -> Self {
        Self::Unknown {
            proxy_type,
            proxy_request,
        }
    }
}

/// Proxy response
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ProxyResponse {
    /// NIP-05 proxy request
    Nip05 {
        /// NIP-05 address to proxy
        address: String,
        /// The response of the NIP-05 request
        response: Value,
    },
    /// Unknown proxy request type
    Unknown {
        /// The type of data being proxied
        proxy_type: String,
        /// The request being proxied
        proxy_request: String,
        /// The response of the request
        response: Value,
    },
}

impl ProxyResponse {
    /// Create new `NIP-05` proxy response
    pub fn new_nip05(address: String, response: Value) -> Self {
        Self::Nip05 { address, response }
    }

    /// Create new `Unknown` proxy response
    pub fn new_unknown(proxy_type: String, proxy_request: String, response: Value) -> Self {
        Self::Unknown {
            proxy_type,
            proxy_request,
            response,
        }
    }
}
