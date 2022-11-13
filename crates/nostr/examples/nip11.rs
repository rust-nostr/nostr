// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

extern crate nostr;

use std::str::FromStr;

use nostr::util::nips::nip11::{self, RelayInformationDocument};
use url::Url;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let relay_url = Url::from_str("https://relay.damus.io")?;

    let info: RelayInformationDocument = nip11::get_relay_information_document(relay_url, None)?;

    println!("{:#?}", info);

    Ok(())
}
