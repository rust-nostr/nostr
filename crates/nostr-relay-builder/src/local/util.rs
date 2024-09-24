// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use tokio::net::TcpListener;

use crate::error::Error;

pub async fn find_available_port() -> nostr::Result<u16, Error> {
    for port in 8000..u16::MAX {
        if port_is_available(port).await {
            return Ok(port);
        }
    }

    Err(Error::NoPortAvailable)
}

#[inline]
pub async fn port_is_available(port: u16) -> bool {
    TcpListener::bind(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, port)))
        .await
        .is_ok()
}
