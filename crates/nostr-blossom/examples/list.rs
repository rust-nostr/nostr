use clap::Parser;
use nostr::prelude::*;
use nostr_blossom::prelude::*;

#[derive(Parser, Debug)]
#[command(author, version, about = "List blob on a Blossom server", long_about = None)]
struct Args {
    /// The server URL to connect to
    #[arg(long)]
    server: String,

    /// The public key to list blobs for
    #[arg(long)]
    pubkey: PublicKey,

    /// Optional private key for authorization (in hex)
    #[arg(long)]
    private_key: Option<SecretKey>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let client = BlossomClient::new(&args.server);

    // Check if a private key was provided and branch accordingly
    if let Some(private_key) = args.private_key {
        // Attempt to create the secret key, propagating error if parsing fails
        let keys = Keys::new(private_key);

        let descriptors = client
            .list_blobs(&args.pubkey, None, None, None, Some(&keys))
            .await?;

        println!("Successfully listed blobs (with auth):");
        for descriptor in descriptors {
            println!("{:?}", descriptor);
        }
    } else {
        let descriptors = client
            .list_blobs(&args.pubkey, None, None, None, None::<&Keys>)
            .await?;

        println!("Successfully listed blobs (without auth):");
        for descriptor in descriptors {
            println!("{:?}", descriptor);
        }
    }

    Ok(())
}
