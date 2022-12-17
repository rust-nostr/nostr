// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::net::SocketAddr;

use tokio::net::TcpStream;
use tokio_socks::tcp::Socks5Stream;
use tokio_socks::IntoTargetAddr;

pub(crate) struct TpcSocks5Stream;

impl TpcSocks5Stream {
    pub async fn connect<'a>(
        proxy: SocketAddr,
        dest: impl IntoTargetAddr<'a>,
    ) -> Result<Socks5Stream<TcpStream>, tokio_socks::Error> {
        Socks5Stream::connect(proxy, dest).await
    }
}
