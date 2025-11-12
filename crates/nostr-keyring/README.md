# Nostr Keyring

Thin wrapper around the system keyring that stores `nostr::Keys` objects without forcing you to handle secret material manually. The crate keeps all serialization in-memory and relies on the OS-provided credential store (macOS Keychain, Windows Credential Manager, Secret Service, etc.).

```rust
use nostr::prelude::*;
use nostr_keyring::NostrKeyring;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let keyring = NostrKeyring::new("my-nostr-app");
    let keys = Keys::generate();

    keyring.set("default", &keys)?;
    let restored = keyring.get("default")?;
    assert_eq!(keys.public_key(), restored.public_key());

    Ok(())
}
```

Enable the `async` feature to offload OS keyring access to a blocking thread pool when running inside async executors:

```rust,no_run
use nostr::prelude::*;
use nostr_keyring::NostrKeyring;

# #[tokio::main]
# async fn main() -> Result<(), Box<dyn std::error::Error>> {
let keyring = NostrKeyring::new("bot");
let keys = keyring.get_async("default").await?;
println!("Using {}", keys.public_key());
# Ok(()) }
```

## Crate Feature Flags

The following crate feature flags are available:

| Feature | Default | Description                               |
|---------|:-------:|-------------------------------------------|
| `async` |   No    | Enable async APIs                         |

Install with `cargo add nostr-keyring --features async` to opt into the Tokio-friendly async helpers.

## Changelog

All notable changes to this library are documented in the [CHANGELOG.md](CHANGELOG.md).

## State

**This library is in an ALPHA state**, things that are implemented generally work but the API will change in breaking ways.

## Donations

`rust-nostr` is free and open-source. This means we do not earn any revenue by selling it. Instead, we rely on your financial support. If you actively use any of the `rust-nostr` libs/software/services, then please [donate](https://rust-nostr.org/donate).

## License

This project is distributed under the MIT software license - see the [LICENSE](../../LICENSE) file for details
