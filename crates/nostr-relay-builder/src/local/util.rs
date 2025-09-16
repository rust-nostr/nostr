// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use nostr::secp256k1::rand::Rng;
use nostr::secp256k1::rand::rngs::OsRng;
use tokio::net::TcpListener;

pub async fn find_available_port() -> u16 {
    let mut rng: OsRng = OsRng;
    loop {
        let port: u16 = rng.gen_range(1024..=u16::MAX);
        if port_is_available(port).await {
            return port;
        }
    }
}

#[inline]
pub async fn port_is_available(port: u16) -> bool {
    TcpListener::bind(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, port)))
        .await
        .is_ok()
}
