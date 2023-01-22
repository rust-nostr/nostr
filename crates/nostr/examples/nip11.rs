// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use nostr::nips::nip11::{self, RelayInformationDocument};
use nostr::url::Url;
use nostr::Result;

fn main() -> Result<()> {
    env_logger::init();

    let relay_url = Url::parse("https://relay.damus.io")?;

    let info: RelayInformationDocument = nip11::get_relay_information_document(relay_url, None)?;

    println!("{:#?}", info);

    Ok(())
}
