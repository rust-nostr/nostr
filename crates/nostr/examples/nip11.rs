// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use nostr::prelude::*;

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let relay_url = Url::parse("wss://relay.damus.io")?;

    let info = RelayInformationDocument::get_blocking(relay_url, None)?;

    println!("{:#?}", info);

    Ok(())
}
