// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use nostr::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let public_key =
        PublicKey::parse("b2d670de53b27691c0c3400225b65c35a26d06093bcc41f48ffc71e0907f9d4a")?;
    // 1. Create a request
    let request = nip05::Nip05Request::new("0xtr@oxtr.dev")?;
    // 2. Use reqwest (or any HTTP client) to fetch the data
    let client = reqwest::Client::new();
    let response = client.get(request.url()).send().await?.text().await?;
    // 3. Verify
    if nip05::verify(&public_key, "0xtr@oxtr.dev", None).await? {
        println!("NIP05 verified");
    } else {
        println!("NIP05 NOT verified");
    }

    let profile_request = nip05::Nip05Request::new("_@fiatjaf.com")?;
    let profile_response = client
        .get(profile_request.url())
        .send()
        .await?
        .text()
        .await?;
    let profile: Nip05Profile = nip05::profile_from_response(&profile_response, &profile_request)?;

    println!("Public key: {}", profile.public_key);
    println!("Relays: {:?}", profile.relays);
    println!("Relays (NIP46): {:?}", profile.nip46);

    // using the simple URL function
    println!("\n--- Alternative approach using get_nip05_url ---");
    let url = nip05::get_nip05_url("test@example.com")?;
    println!("Would fetch from: {}", url);
    // You can now use any HTTP client to fetch from this URL

    Ok(())
}
