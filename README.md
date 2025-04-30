# Nostr

## Project structure

The project is split up into several crates in the `crates/` directory:

* Libraries:
    * [**nostr**](./crates/nostr): Rust implementation of Nostr protocol
    * [**nostr-blossom**](./crates/nostr-blossom): A library for interacting with the Blossom protocol
    * [**nostr-connect**](./crates/nostr-connect): Nostr Connect (NIP46)
    * [**nostr-database**](./crates/nostr-database): Database for Nostr apps
        * [**nostr-lmdb**](./crates/nostr-lmdb): LMDB storage backend
        * [**nostr-ndb**](./crates/nostr-ndb): [nostrdb](https://github.com/damus-io/nostrdb) storage backend
        * [**nostr-sqldb**](./crates/nostr-sqldb): SQL storage backends (PostgreSQL, MySQL and SQLite)
        * [**nostr-indexeddb**](./crates/nostr-indexeddb): IndexedDB storage backend
    * [**nostr-mls**](./crates/nostr-mls): A library for implmenting NIP-EE MLS messaging
    * [**nostr-mls-storage**](./crates/nostr-mls-storage): Storage traits for using MLS messaging
        * [**nostr-mls-memory-storage**](./crates/nostr-mls-memory-storage): In-memory storage for nostr-mls
        * [**nostr-mls-sqlite-storage**](./crates/nostr-mls-sqlite-storage): Sqlite storage for nostr-mls
    * [**nostr-keyring**](./crates/nostr-keyring): Nostr Keyring
    * [**nostr-relay-pool**](./crates/nostr-relay-pool): Nostr Relay Pool
    * [**nostr-sdk**](./crates/nostr-sdk): High level client library
    * [**nwc**](./crates/nwc): Nostr Wallet Connect (NWC) client
* Binaries (tools):
    * [**nostr-cli**](./crates/nostr-cli): Nostr CLI

> Note: this repository contains the Rust codebase.
> There are several other projects (i.e., bindings)
> which are maintained in other repositories <https://rust-nostr.org/projects>.

### Embedded

**nostr** crate can be used in [`no_std`](https://docs.rust-embedded.org/book/intro/no-std.html) environments.
Check the example in the [`embedded/`](./crates/nostr/examples/embedded) directory.

## Book

Learn more about `rust-nostr` at <https://rust-nostr.org>.

## State

**These libraries are in ALPHA state**, things that are implemented generally work but the API will change in breaking ways.

## Donations

`rust-nostr` is free and open-source. This means we do not earn any revenue by selling it. Instead, we rely on your financial support. If you actively use any of the `rust-nostr` libs/software/services, then please [donate](https://rust-nostr.org/donate).

## License

This project is distributed under the MIT software license - see the [LICENSE](LICENSE) file for details
