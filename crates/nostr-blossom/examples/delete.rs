use std::error::Error;
use std::str::FromStr;

use clap::Parser;
use nostr::hashes::sha256;
use nostr::key::SecretKey;
use nostr::Keys;
use nostr_blossom::client::BlossomClient;

#[derive(Parser, Debug)]
#[command(author, version, about = "Delete a blob from a Blossom server", long_about = None)]
struct Args {
    /// The server URL to connect to
    #[arg(long)]
    server: String,

    /// The SHA256 hash of the blob to delete (in hex)
    #[arg(long)]
    sha256: String,

    /// Optional private key for signing the deletion (in hex)
    #[arg(long, value_name = "PRIVATE_KEY")]
    private_key: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let client = BlossomClient::new(&args.server);

    // Create signer keys using the given private key
    let keys = Keys::new(SecretKey::from_hex(&args.private_key)?);

    println!("Attempting to delete blob with SHA256: {}", args.sha256);

    let blob_hash = sha256::Hash::from_str(&args.sha256)?;

    match client.delete_blob(blob_hash, None, &keys).await {
        Ok(()) => println!("Blob deleted successfully."),
        Err(e) => eprintln!("Failed to delete blob: {}", e),
    }

    Ok(())
}
