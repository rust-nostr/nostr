use std::error::Error;
use std::fs;

use clap::Parser;
use nostr::key::SecretKey;
use nostr::Keys;
use nostr_blossom::client::BlossomClient;

#[derive(Parser, Debug)]
#[command(author, version, about = "Upload a blob to a Blossom server", long_about = None)]
struct Args {
    /// The server URL to connect to
    #[arg(long)]
    server: String,

    /// Path to the file to upload
    #[arg(long)]
    file: String,

    /// Optional content type (e.g., "application/octet-stream")
    #[arg(long)]
    content_type: Option<String>,

    /// Optional private key for signing the upload (in hex)
    #[arg(long)]
    private_key: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let client = BlossomClient::new(&args.server);

    // Read file data from the specified file path.
    let data = fs::read(&args.file)?;

    // Use the provided content type or default to "application/octet-stream"
    let content_type = args
        .content_type
        .clone()
        .or_else(|| Some("application/octet-stream".to_string()));

    // Create signer keys.
    // If a private key is provided, try to use it; otherwise generate a new key.
    let keys = match args.private_key {
        Some(private_key) => match SecretKey::from_hex(private_key.as_str()) {
            Ok(secret_key) => Keys::new(secret_key),
            Err(e) => {
                eprintln!(
                    "Failed to parse private key: {}. Using generated key instead.",
                    e
                );
                Keys::generate()
            }
        },
        None => Keys::generate(),
    };

    match client
        .upload_blob(data, content_type, None, Some(&keys))
        .await
    {
        Ok(descriptor) => {
            println!("Successfully uploaded blob: {:?}", descriptor);
        }
        Err(err) => {
            eprintln!("Failed to upload blob: {}", err);
        }
    }

    Ok(())
}
