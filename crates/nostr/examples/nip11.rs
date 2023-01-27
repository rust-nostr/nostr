// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use nostr::prelude::*;

fn main() -> Result<()> {
    env_logger::init();

    let relay_url = Url::parse("https://relay.damus.io")?;

    let info = RelayInformationDocument::get(relay_url, None)?;

    println!("{:#?}", info);

    Ok(())
}
