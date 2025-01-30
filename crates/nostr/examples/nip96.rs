// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use nostr::prelude::*;

const FILE: &[u8] = include_bytes!("../../../LICENSE");

#[tokio::main]
async fn main() -> Result<()> {
    let keys = Keys::parse("nsec1j4c6269y9w0q2er2xjw8sv2ehyrtfxq3jwgdlxj6qfn8z4gjsq5qfvfk99")?;

    let server_url: Url = Url::parse("https://NostrMedia.com")?;

    let config: ServerConfig = nip96::get_server_config(server_url, None).await?;

    let file_data: Vec<u8> = FILE.to_vec();

    // Upload
    let url: Url = nip96::upload_data(&keys, &config, file_data, None, None).await?;
    println!("File uploaded: {url}");

    Ok(())
}
