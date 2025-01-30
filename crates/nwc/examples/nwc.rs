// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::str::FromStr;

use nwc::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let mut nwc_uri_string = String::new();

    println!("Please enter a NWC string");
    std::io::stdin()
        .read_line(&mut nwc_uri_string)
        .expect("Failed to read line");

    // Parse URI and compose NWC client
    let uri: NostrWalletConnectURI =
        NostrWalletConnectURI::from_str(&nwc_uri_string).expect("Failed to parse NWC URI");
    let nwc: NWC = NWC::new(uri);

    // Get balance
    let balance = nwc.get_balance().await?;
    println!("Balance: {balance} msat");

    let request: PayInvoiceRequest = PayInvoiceRequest::new("<invoice>");
    let response = nwc.pay_invoice(request).await?;
    println!("Response: {response:?}");

    Ok(())
}
