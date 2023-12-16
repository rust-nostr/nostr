// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! WASM

use futures_util::stream::{SplitSink, SplitStream};
use futures_util::StreamExt;
use url_fork::Url;
use wasm_ws::{WebSocket, WsErr, WsMessage, WsStream};

type Sink = SplitSink<WsStream, WsMessage>;
type Stream = SplitStream<WsStream>;

pub async fn connect(url: &Url) -> Result<(Sink, Stream), WsErr> {
    let (_ws, stream) = WebSocket::connect(url).await?;
    Ok(stream.split())
}
