use std::fs;
use std::path::PathBuf;

use clap::Parser;
use nostr::prelude::*;
use nostr_blossom::prelude::*;

#[derive(Parser, Debug)]
#[command(author, version, about = "Upload a blob to a Blossom server", long_about = None)]
struct Args {
    /// The server URL to connect to
    #[arg(long)]
    server: String,

    /// Path to the file to upload
    #[arg(long)]
    file: PathBuf,

    /// Optional content type (e.g., "application/octet-stream")
    #[arg(long)]
    content_type: Option<String>,

    /// Optional private key for signing the upload
    #[arg(long)]
    private_key: Option<SecretKey>,
}

#[tokio::main]
async fn main() -> Result<()> {
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
        Some(private_key) => Keys::new(private_key),
        None => Keys::generate(),
    };

    match client
        .upload_blob(data, content_type, None, Some(&keys))
        .await
    {
        Ok(descriptor) => {
            println!("Successfully uploaded blob: {:?}", descriptor);
        }
        Err(e) => {
            eprintln!("{e}");
        }
    }

    Ok(())
}
