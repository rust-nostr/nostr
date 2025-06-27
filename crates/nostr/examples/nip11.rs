// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use nostr::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let relay_url = Url::parse("wss://relay.damus.io")?;

    // Convert WebSocket URL to HTTP and fetch relay information
    let http_url = RelayInformationDocument::get_http_url_from_ws(&relay_url)?;
    let client = reqwest::Client::new();
    let response = client
        .get(&http_url)
        .header("Accept", "application/nostr+json")
        .send()
        .await?;
    let json = response.text().await?;
    let info = RelayInformationDocument::parse(&json)?;

    println!("{:#?}", info);

    Ok(())
}
