// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use nostr::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let relay_url = Url::parse("wss://relay.damus.io")?;

    // 1. Create the request
    let request = nip11::Nip11Request::new(relay_url)?;

    // 2. Use reqwest (or any HTTP client) to fetchdata
    let client = reqwest::Client::new();
    let mut req_builder = client.get(request.url());

    // 3. Add the required headers
    for (name, value) in request.headers() {
        req_builder = req_builder.header(name, value);
    }

    // 4. Send the request and get the response
    let response = req_builder.send().await?.text().await?;

    // 5. Parse the relay information document
    let info = RelayInformationDocument::from_response(&response)?;

    println!("{:#?}", info);

    // Example showing flexibility - using the simple URL function
    println!("\n--- Alternative approach using get_relay_info_url ---");
    let relay_url2 = Url::parse("wss://relay.snort.social")?;
    let info_url = nip11::get_relay_info_url(relay_url2)?;
    println!("Would fetch relay info from: {}", info_url);
    println!("Remember to include Accept: application/nostr+json header");

    Ok(())
}
