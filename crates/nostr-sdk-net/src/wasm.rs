// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! WASM

use futures_util::stream::{SplitSink, SplitStream};
use futures_util::StreamExt;
use url::Url;

type Sink = SplitSink<ws_stream_wasm::WsStream, ws_stream_wasm::WsMessage>;
type Stream = SplitStream<ws_stream_wasm::WsStream>;

pub async fn connect(url: &Url) -> Result<(Sink, Stream), ws_stream_wasm::WsErr> {
    let (_ws, stream) = ws_stream_wasm::WsMeta::connect(url, None).await?;
    Ok(stream.split())
}
