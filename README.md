# Nostr

## Project structure

The project is split up into several crates in the `crates/` directory:

* Libraries:
    * [**nostr**](./crates/nostr/): Rust implementation of Nostr protocol.
    * [**nostr-database**](./crates/nostr-database/): Database for Nostr apps
        * [**nostr-rocksdb**](./crates/nostr-rocksdb/): RocksDB Storage backend for Nostr apps
        * [**nostr-sqlite**](./crates/nostr-sqlite/): SQLite Storage backend for Nostr apps
        * [**nostr-indexeddb**](./crates/nostr-indexeddb/): IndexedDB Storage backend for Nostr apps
    * [**nostr-sdk**](./crates/nostr-sdk/): High level client library.
    * [**nostr-sdk-net**](./crates/nostr-sdk-net/): Network library for [**nostr-sdk**](./crates/nostr-sdk/)
* Binaries (tools):
    * [**nostr-cli**](./crates/nostr-cli/): Nostr CLI

### Bindings

**nostr** and **nostr-sdk** crates can be embedded inside other environments, like Swift, Kotlin, Python and JavaScript. 
Please, explore the [`bindings/`](./bindings/) directory to learn more.

### Embedded

**nostr** crate can be used in [`no_std`](https://docs.rust-embedded.org/book/intro/no-std.html) environments. 
Check the example in the [`embedded/`](./crates/nostr/examples/embedded/) directory.

### Available packages

* **nostr**:
    * Rust: https://crates.io/crates/nostr
    * Python: https://pypi.org/project/nostr-protocol
    * Kotlin: [`io.github.rust-nostr:nostr`](https://central.sonatype.com/artifact/io.github.rust-nostr/nostr/)
    * Swift: https://github.com/rust-nostr/nostr-swift
    * JavaScript: https://www.npmjs.com/package/@rust-nostr/nostr
* **nostr-sdk**:
    * Rust: https://crates.io/crates/nostr-sdk
    * Python: https://pypi.org/project/nostr-sdk
    * Kotlin: [`io.github.rust-nostr:nostr-sdk`](https://central.sonatype.com/artifact/io.github.rust-nostr/nostr-sdk/)
    * Swift: https://github.com/rust-nostr/nostr-sdk-swift
    * JavaScript: https://www.npmjs.com/package/@rust-nostr/nostr-sdk

## State

**These libraries are in ALPHA state**, things that are implemented generally work but the API will change in breaking ways.

## License

This project is distributed under the MIT software license - see the [LICENSE](LICENSE) file for details

## Donations

⚡ Tips: <https://getalby.com/p/yuki>

⚡ Lightning Address: yuki@getalby.com