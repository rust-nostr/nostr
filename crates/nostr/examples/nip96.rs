// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

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
    let server_url: Url = Url::parse("https://nostr.media")?;

    // Step 1: Get server configuration URL
    let config_url = nip96::get_server_config_url(&server_url)?;
    println!("Config URL: {}", config_url);

    // Mock server config response
    let config_json = r#"{
        "api_url": "https://nostr.media/api/v1/nip96/upload",
        "download_url": "https://nostr.media"
    }"#;

    let config = nip96::ServerConfig::from_json(config_json)?;
    println!("Upload endpoint: {}", config.api_url);

    // Step 2: Prepare upload request
    let upload_request = nip96::UploadRequest::new(&keys, &config, FILE).await?;
    println!("Upload URL: {}", upload_request.url());
    println!("Authorization: {}", upload_request.authorization());

    // Step 3: Mock upload response
    let upload_response_json = r#"{
        "status": "success",
        "message": "Upload successful",
        "nip94_event": {
            "tags": [["url", "https://nostr.media/file123.png"]]
        }
    }"#;

    // Parse response and extract URL
    let upload_response = nip96::UploadResponse::from_json(upload_response_json)?;
    match upload_response.download_url() {
        Ok(url) => println!("File would be available at: {url}"),
        Err(e) => println!("Upload simulation failed: {e}"),
    }

    println!("\n--- I/O-free NIP96 Demo Complete ---");
    println!("This example shows how to use NIP96 without any specific HTTP client.");
    println!("Users can now choose reqwest, ureq, curl, or any HTTP implementation!");

    Ok(())
}
