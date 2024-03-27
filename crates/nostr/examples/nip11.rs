// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use nostr::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let relay_url = Url::parse("wss://relay.damus.io")?;

    let info = RelayInformationDocument::get(relay_url, None).await?;

    println!("{:#?}", info);

    Ok(())
}
