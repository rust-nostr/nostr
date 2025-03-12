use std::error::Error;
use std::str::FromStr;

use clap::Parser;
use nostr::key::SecretKey;
use nostr::{Keys, PublicKey};
use nostr_blossom::client::BlossomClient;

#[derive(Parser, Debug)]
#[command(author, version, about = "List blob on a Blossom server", long_about = None)]
struct Args {
    /// The server URL to connect to
    #[arg(long)]
    server: String,

    /// The public key to list blobs for (in hex format)
    #[arg(long)]
    pubkey: String,

    /// Optional private key for authorization (in hex)
    #[arg(long)]
    private_key: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let client = BlossomClient::new(&args.server);

    let pubkey = PublicKey::from_str(&args.pubkey)?;

    // Check if a private key was provided and branch accordingly
    if let Some(private_key_str) = args.private_key {
        // Attempt to create the secret key, propagating error if parsing fails
        let secret_key = SecretKey::from_hex(&private_key_str)?;
        let keys = Keys::new(secret_key);

        let descriptors = client
            .list_blobs(&pubkey, None, None, None, Some(&keys))
            .await?;

        println!("Successfully listed blobs (with auth):");
        for descriptor in descriptors {
            println!("{:?}", descriptor);
        }
    } else {
        let descriptors = client
            .list_blobs(&pubkey, None, None, None, None::<&Keys>)
            .await?;

        println!("Successfully listed blobs (without auth):");
        for descriptor in descriptors {
            println!("{:?}", descriptor);
        }
    }

    Ok(())
}
