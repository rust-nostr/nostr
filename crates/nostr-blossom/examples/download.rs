use std::error::Error;
use std::fs;
use std::str::FromStr;

use clap::Parser;
use nostr::hashes::sha256;
use nostr_blossom::client::BlossomClient;

#[derive(Parser, Debug)]
#[command(author, version, about = "Download a blob from a Blossom server", long_about = None)]
struct Args {
    /// The server URL to connect to
    #[arg(long)]
    server: String,

    /// SHA256 hash of the blob to download
    #[arg(long)]
    sha256: String,

    /// Private key to use for authorization (in hex)
    #[arg(long)]
    private_key: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    // Initialize the client.
    let client = BlossomClient::new(&args.server);

    // Convert the provided SHA256 string into a hash.
    let blob_sha = sha256::Hash::from_str(&args.sha256)?;

    // Parse the private key.
    let keypair = nostr::Keys::parse(&args.private_key)?;

    // Download the blob with optional authorization.
    match client.get_blob(blob_sha, None, None, Some(&keypair)).await {
        Ok(blob) => {
            println!("Successfully downloaded blob with {} bytes", blob.len());
            let file_name = format!("{}", blob_sha);
            fs::write(&file_name, &blob)?;
            println!("Blob saved as {}", file_name);
        }
        Err(err) => {
            eprintln!("Failed to download blob: {}", err);
        }
    }

    Ok(())
}
