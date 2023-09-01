// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! WASM

use futures_util::stream::{SplitSink, SplitStream};
use futures_util::StreamExt;
use url_fork::Url;
use ws_stream_wasm::{WsErr, WsMessage, WsMeta, WsStream};

type Sink = SplitSink<WsStream, WsMessage>;
type Stream = SplitStream<WsStream>;

pub async fn connect(url: &Url) -> Result<(Sink, Stream), WsErr> {
    let (_ws, stream) = WsMeta::connect(url, None).await?;
    Ok(stream.split())
}
