// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use nostr::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let url = Url::parse("https://relay.damus.io")?;

    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .header("Accept", "application/nostr+json")
        .send()
        .await?;
    let json: String = response.text().await?;

    let info = RelayInformationDocument::from_json(&json)?;

    println!("{info:#?}");

    Ok(())
}
