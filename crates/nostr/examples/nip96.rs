// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use nostr::nips::nip96::{HttpClient, IoError};
use nostr::prelude::*;

const FILE: &[u8] = &[
    137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 1, 0, 0, 0, 1, 8, 6, 0,
    0, 0, 31, 21, 196, 137, 0, 0, 0, 1, 115, 82, 71, 66, 0, 174, 206, 28, 233, 0, 0, 0, 4, 103, 65,
    77, 65, 0, 0, 177, 143, 11, 252, 97, 5, 0, 0, 0, 9, 112, 72, 89, 115, 0, 0, 28, 35, 0, 0, 28,
    35, 1, 199, 111, 168, 100, 0, 0, 0, 12, 73, 68, 65, 84, 8, 29, 99, 248, 255, 255, 63, 0, 5,
    254, 2, 254, 135, 150, 28, 0, 0, 0, 0, 73, 69, 78, 68, 174, 66, 96, 130,
];

#[tokio::main]
async fn main() -> Result<()> {
    let keys = Keys::parse("nsec1j4c6269y9w0q2er2xjw8sv2ehyrtfxq3jwgdlxj6qfn8z4gjsq5qfvfk99")?;
    let server_url: Url = Url::parse("https://NostrMedia.com")?;

    // Step 1: Get server configuration
    let config_request = nip96::ServerConfigRequest::new(server_url)?;

    let client = reqwest::Client::new();
    let config_response = client
        .get(config_request.url())
        .send()
        .await?
        .text()
        .await?;
    let config: ServerConfig = nip96::server_config_from_response(&config_response)?;

    println!("Server config loaded from: {}", config_request.url());
    println!("Upload endpoint: {}", config.api_url);

    // Step 2: Prepare file upload
    let file_data: &[u8] = FILE;

    // Create upload request
    let upload_request = nip96::UploadRequest::new(&keys, &config, file_data).await?;

    // Step 3: Create multipart form (users handle this with their preferred method)
    let form_file_part = reqwest::multipart::Part::bytes(file_data.to_vec())
        .file_name("test.png")
        .mime_str("image/png")?;

    let form = reqwest::multipart::Form::new().part("file", form_file_part);

    // Step 4: Upload file using reqwest
    let upload_response = client
        .post(upload_request.url())
        .header("Authorization", upload_request.authorization())
        .multipart(form)
        .send()
        .await?
        .text()
        .await?;

    // Step 5: Parse response and extract URL
    match nip96::upload_response_to_url(&upload_response) {
        Ok(url) => println!("File uploaded successfully: {url}"),
        Err(e) => println!("Upload failed: {e}"),
    }

    // Example showing flexibility - you can use any HTTP client
    println!("\n--- This approach works with any HTTP client! ---");
    println!(
        "1. Get config URL: {}",
        nip96::get_server_config_url(&server_url)?
    );
    println!("2. Fetch config with your HTTP client");
    println!("3. Parse with nip96::server_config_from_response()");
    println!("4. Create upload request with UploadRequest::new()");
    println!("5. Upload with your HTTP client + multipart form");
    println!("6. Parse response with nip96::upload_response_to_url()");

    Ok(())
}
