use std::fs;

use clap::Parser;
use nostr::hashes::sha256::Hash as Sha256Hash;
use nostr::prelude::*;
use nostr_blossom::prelude::*;

#[derive(Parser, Debug)]
#[command(author, version, about = "Download a blob from a Blossom server", long_about = None)]
struct Args {
    /// The server URL to connect to
    #[arg(long)]
    server: Url,

    /// SHA256 hash of the blob to download
    #[arg(long)]
    sha256: Sha256Hash,

    /// Private key to use for authorization
    #[arg(long)]
    private_key: SecretKey,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize the client.
    let client = BlossomClient::new(args.server);

    // Parse the private key.
    let keypair = Keys::new(args.private_key);

    // Download the blob with optional authorization.
    match client
        .get_blob(args.sha256, None, None, Some(&keypair))
        .await
    {
        Ok(blob) => {
            println!("Successfully downloaded blob with {} bytes", blob.len());
            let file_name = format!("{}", args.sha256);
            fs::write(&file_name, &blob)?;
            println!("Blob saved as {}", file_name);
        }
        Err(e) => {
            eprintln!("{e}");
        }
    }

    Ok(())
}
