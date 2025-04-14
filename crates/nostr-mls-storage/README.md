# Nostr MLS Storage

This crate provides an abstraction for the storage layer that MLS requires.

## `NostrMlsStorageProvider` Trait

THis library contains the `NostrMlsStorageProvider` trait. You can use the [default backends](#default-backends) or implement your own.

## Default Backends

- Memory (RAM) - [nostr-mls-memory-storage](../nostr-mls-memory-storage)
- Sqlite (Native) - [nostr-mls-sqlite-storage](../nostr-mls-sqlite-storage/)

## State

**This library is in an ALPHA state**, things that are implemented generally work but the API will change in breaking ways.

## Donations

`rust-nostr` is free and open-source. This means we do not earn any revenue by selling it. Instead, we rely on your financial support. If you actively use any of the `rust-nostr` libs/software/services, then please [donate](https://rust-nostr.org/donate).

## License

This project is distributed under the MIT software license - see the [LICENSE](../../LICENSE) file for details
