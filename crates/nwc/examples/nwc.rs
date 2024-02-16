// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::str::FromStr;

use nwc::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let mut nwc_uri_string = String::new();
    let mut invoice = String::new();

    println!("Please enter a NWC string");
    std::io::stdin()
        .read_line(&mut nwc_uri_string)
        .expect("Failed to read line");

    println!("Please enter a BOLT 11 invoice");
    std::io::stdin()
        .read_line(&mut invoice)
        .expect("Failed to read line");

    invoice = String::from(invoice.trim());

    // Parse URI and compose NWC client
    let uri = NostrWalletConnectURI::from_str(&nwc_uri_string).expect("Failed to parse NWC URI");
    let nwc = NWC::new(uri).await?;

    // Pay invoice
    nwc.send_payment(invoice).await?;

    Ok(())
}
