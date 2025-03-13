// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use nostr::prelude::*;

const FILE: &[u8] = &[
    137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 1, 0, 0, 0, 1, 8, 6, 0,
    0, 0, 31, 21, 196, 137, 0, 0, 0, 1, 115, 82, 71, 66, 0, 174, 206, 28, 233, 0, 0, 0, 4, 103, 65,
    77, 65, 0, 0, 177, 143, 11, 252, 97, 5, 0, 0, 0, 9, 112, 72, 89, 115, 0, 0, 28, 35, 0, 0, 28,
    35, 1, 199, 111, 168, 100, 0, 0, 0, 12, 73, 68, 65, 84, 8, 29, 99, 248, 255, 255, 63, 0, 5,
    254, 2, 254, 135, 150, 28, 0, 0, 0, 0, 73, 69, 78, 68, 174, 66, 96, 130,
];

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
