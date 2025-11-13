# Nostr HTTP File Storage client (NIP-96)

Client for [NIP-96](https://github.com/nostr-protocol/nips/blob/master/96.md) working with servers. 
Handles discovery of `nip96.json`, authenticated uploads, and returns the download URL you can embed inside events.

```rust,no_run
use nostr_http_file_storage::prelude::*;

# #[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = NostrHttpFileStorageClient::new();
    let server = Url::parse("https://files.example.com")?;

    // Fetch nip96.json to learn limits and auth requirements
    let config = client.get_server_config(&server).await?;

    let signer = Keys::parse("<my-key>")?;
    let download_url = client
        .upload(
            &signer,
            &config,
            b"hello nostr".to_vec(),
            Some("text/plain"),
        )
        .await?;

    println!("File available at {download_url}");
    Ok(())
}
```

## Crate Feature Flags

The following crate feature flags are available:

| Feature | Default | Description                    |
|---------|:-------:|--------------------------------|
| `socks` |   No    | Enable support for socks proxy |

## Changelog

All notable changes to this library are documented in the [CHANGELOG.md](CHANGELOG.md).

## State

**This library is in an ALPHA state**, things that are implemented generally work but the API will change in breaking ways.

## Donations

`rust-nostr` is free and open-source. This means we do not earn any revenue by selling it. Instead, we rely on your financial support. If you actively use any of the `rust-nostr` libs/software/services, then please [donate](https://rust-nostr.org/donate).

## License

This project is distributed under the MIT software license - see the [LICENSE](../../LICENSE) file for details
