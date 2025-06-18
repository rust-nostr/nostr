# Nostr MLS SQLCipher Storage

SQLCipher MLS storage backend for nostr apps

## State

**This library is in an ALPHA state**, things that are implemented generally work but the API will change in breaking ways.

## Features

This crate provides two cargo features to control how SQLCipher and its crypto backend are packaged:

| Feature | Description |
| ------- | ----------- |
| default (`bundled-sqlcipher-vendored-openssl`) | Builds SQLCipher from source and statically links it together with a vendored OpenSSL via the `openssl-sys` crate. Ideal for fully self-contained binaries without system dependencies. |
| `bundled-sqlcipher` | Builds SQLCipher from source but relies on the system OpenSSL / LibreSSL / Security.framework for the crypto implementation. |

### Build examples

```bash
# 1. Default build: SQLCipher + vendored (statically linked) OpenSSL
cargo build

# 2. Use system crypto library, still bundle SQLCipher
cargo build --no-default-features --features bundled-sqlcipher
```

## Donations

`rust-nostr` is free and open-source. This means we do not earn any revenue by selling it. Instead, we rely on your financial support. If you actively use any of the `rust-nostr` libs/software/services, then please [donate](https://rust-nostr.org/donate).

## License

This project is distributed under the MIT software license - see the [LICENSE](../../LICENSE) file for details
