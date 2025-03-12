use std::error::Error;
use std::fs;
use std::path::PathBuf;

use clap::Parser;
use nostr::hashes::{sha256, Hash};
use nostr::key::SecretKey;
use nostr::Keys;
use nostr_blossom::client::BlossomClient;

/// Integration test for various Blossom operations: upload, check (HEAD), list, download, and delete.
#[derive(Parser, Debug)]
#[command(author, version, about = "Run several operations against a Blossom server for demonstration and testing", long_about = None)]
struct Args {
    /// The Blossom server URL
    #[arg(long)]
    server: String,

    /// Optional file path to a blob to upload. If omitted, a small test blob is used.
    #[arg(long)]
    file: Option<PathBuf>,

    /// Private key to use for signing (in hex)
    #[arg(long, value_name = "PRIVATE_KEY")]
    private_key: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Parse command line arguments.
    let args = Args::parse();

    // Create the Blossom client.
    let client = BlossomClient::new(&args.server);
    // Create signer keys from the provided private key.
    let keys = Keys::new(SecretKey::from_hex(args.private_key.as_str())?);

    // Read the blob data from a file if provided, otherwise use default test data.
    let data = if let Some(file_path) = args.file {
        println!("Reading blob data from file: {:?}", file_path);
        fs::read(file_path)?
    } else {
        let default_blob = b"Test blob data from integration_test";
        println!(
            "No file provided. Using default test data: {:?}",
            String::from_utf8_lossy(default_blob)
        );
        default_blob.to_vec()
    };

    // Compute SHA256 hash of the blob data.
    let blob_hash = sha256::Hash::hash(&data);
    let blob_hash_hex = blob_hash.to_string();
    println!("\nBlob SHA256: {}", blob_hash_hex);

    // 1. Upload Blob
    println!("\n[1] Uploading blob...");
    // Use a basic content type
    let content_type = Some("application/octet-stream".to_string());
    let descriptor = client
        .upload_blob(data.clone(), content_type, None, Some(&keys))
        .await?;
    println!("Uploaded BlobDescriptor: {:#?}", descriptor);

    // 2. Check blob existence using HEAD (has_blob method)
    println!("\n[2] Checking blob existence via HEAD request...");
    let exists = client.has_blob(blob_hash, None, Some(&keys)).await?;
    println!("has_blob result: {}", exists);

    // 3. List blobs for the pubkey
    println!("\n[3] Listing blobs for public key...");
    let pubkey = keys.public_key();
    let blobs = client
        .list_blobs::<Keys>(&pubkey, None, None, None, Some(&keys))
        .await?;
    println!("List Blobs results:");
    for blob in blobs.iter() {
        println!(" - {:?}", blob);
    }

    // 4. Download blob and compare hash
    println!("\n[4] Downloading blob...");

    let downloaded_data: Vec<u8> = client.get_blob(blob_hash, None, None, Some(&keys)).await?;
    let downloaded_hash = sha256::Hash::hash(&downloaded_data);
    println!("Downloaded blob hash: {}", downloaded_hash);
    if downloaded_hash == blob_hash {
        println!("Downloaded blob hash matches the original.");
    } else {
        println!(
            "Hash mismatch! Original: {}  Downloaded: {}",
            blob_hash, downloaded_hash
        );
    }

    // 5. Delete blob
    println!("\n[5] Deleting blob...");
    client.delete_blob(blob_hash, None, &keys).await?;
    println!("Blob deleted successfully.");

    // Final check: verify deletion using HEAD
    let exists_after = client.has_blob(blob_hash, None, Some(&keys)).await?;
    if !exists_after {
        println!("Verified: Blob no longer exists on the server.");
    } else {
        println!("Warning: Blob still exists on the server.");
    }

    println!("\nIntegration test complete.");
    Ok(())
}
