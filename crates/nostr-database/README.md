# Nostr Database

Database abstraction and in-memory implementation for nostr apps

## Nostr Database Trait

This library contains the `NostrDatabase` and `NostrDatabaseExt` traits. You can use the [default backends](#default-backends) or implement your one (like PostgreSQL, ...).

## Default backends

* Memory (RAM, both native and web), available in this library
* LMDB (native), available at [`nostr-lmdb`](https://crates.io/crates/nostr-lmdb)
* [nostrdb](https://github.com/damus-io/nostrdb) (native), available at [`nostr-ndb`](https://crates.io/crates/nostr-ndb)
* IndexedDB (web), available at [`nostr-indexeddb`](https://crates.io/crates/nostr-indexeddb)

## Crate Feature Flags

The following crate feature flags are available:

| Feature   | Default | Description                                            |
|-----------|:-------:|--------------------------------------------------------|
| `flatbuf` |   No    | Enable `flatbuffers` de/serialization for nostr events |

## State

**This library is in an ALPHA state**, things that are implemented generally work but the API will change in breaking ways.

## Donations

`rust-nostr` is free and open-source. This means we do not earn any revenue by selling it. Instead, we rely on your financial support. If you actively use any of the `rust-nostr` libs/software/services, then please [donate](https://rust-nostr.org/donate).

## License

This project is distributed under the MIT software license - see the [LICENSE](../../LICENSE) file for details
