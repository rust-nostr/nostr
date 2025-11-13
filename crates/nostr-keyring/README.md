# Nostr Keyring

Thin wrapper around the system keyring that stores `Keys` objects without forcing you to handle secret material manually. 
The crate keeps all serialization in-memory and relies on the OS-provided credential store (macOS Keychain, Windows Credential Manager, Secret Service, etc.).

## Getting started

```rust,no_run
use nostr_keyring::prelude::*;

fn main() -> Result<()> {
    let keyring = NostrKeyring::new("my-nostr-app");
    
    // Save a key
    let keys = Keys::generate();
    keyring.set("example", &keys)?;
    
    // Get it
    let restored: Keys = keyring.get("example")?;
    
    assert_eq!(keys.public_key(), restored.public_key());

    Ok(())
}
```

More examples can be found in the [examples directory](./examples).

## Crate Feature Flags

The following crate feature flags are available:

| Feature | Default | Description                               |
|---------|:-------:|-------------------------------------------|
| `async` |   No    | Enable async APIs                         |

## Changelog

All notable changes to this library are documented in the [CHANGELOG.md](CHANGELOG.md).

## State

**This library is in an ALPHA state**, things that are implemented generally work but the API will change in breaking ways.

## Donations

`rust-nostr` is free and open-source. This means we do not earn any revenue by selling it. Instead, we rely on your financial support. If you actively use any of the `rust-nostr` libs/software/services, then please [donate](https://rust-nostr.org/donate).

## License

This project is distributed under the MIT software license - see the [LICENSE](../../LICENSE) file for details
