use clap::Parser;
use nostr::hashes::sha256::Hash as Sha256Hash;
use nostr::prelude::*;
use nostr_blossom::prelude::*;

#[derive(Parser, Debug)]
#[command(author, version, about = "Delete a blob from a Blossom server", long_about = None)]
struct Args {
    /// The server URL to connect to
    #[arg(long)]
    server: String,

    /// The SHA256 hash of the blob to delete (in hex)
    #[arg(long)]
    sha256: Sha256Hash,

    /// Optional private key for signing the deletion
    #[arg(long, value_name = "PRIVATE_KEY")]
    private_key: SecretKey,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let client = BlossomClient::new(&args.server);

    // Create signer keys using the given private key
    let keys = Keys::new(args.private_key);

    println!("Attempting to delete blob with SHA256: {}", args.sha256);

    match client.delete_blob(args.sha256, None, &keys).await {
        Ok(()) => println!("Blob deleted successfully."),
        Err(e) => eprintln!("{e}"),
    }

    Ok(())
}
