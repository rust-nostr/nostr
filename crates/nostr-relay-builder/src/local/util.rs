// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::net::{IpAddr, SocketAddr};

use nostr::rand::rngs::OsRng;
use nostr::rand::{Rng, TryRngCore};
use tokio::net::TcpListener;

pub async fn find_available_port(ip: IpAddr) -> u16 {
    let mut rng = OsRng.unwrap_err();
    loop {
        let port: u16 = rng.random_range(1024..=u16::MAX);
        if port_is_available(ip, port).await {
            return port;
        }
    }
}

#[inline]
pub async fn port_is_available(ip: IpAddr, port: u16) -> bool {
    TcpListener::bind(SocketAddr::new(ip, port)).await.is_ok()
}
