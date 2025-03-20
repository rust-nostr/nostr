// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use nostr::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let keys = Keys::parse("nsec1j4c6269y9w0q2er2xjw8sv2ehyrtfxq3jwgdlxj6qfn8z4gjsq5qfvfk99")?;

    let server_url: Url = Url::parse("https://example.com")?;
    let method = HttpMethod::GET;

    let auth = HttpData::new(server_url, method)
        .to_authorization(&keys)
        .await?;

    println!("{auth}");

    Ok(())
}
