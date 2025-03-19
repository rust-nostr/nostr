# Nostr

## Project structure

The project is split up into several crates in the `crates/` directory:

* Libraries:
    * [**nostr**](./crates/nostr): Rust implementation of Nostr protocol
    * [**nostr-connect**](./crates/nostr-connect): Nostr Connect (NIP46)
    * [**nostr-database**](./crates/nostr-database): Database for Nostr apps
        * [**nostr-lmdb**](./crates/nostr-lmdb): LMDB storage backend
        * [**nostr-ndb**](./crates/nostr-ndb): [nostrdb](https://github.com/damus-io/nostrdb) storage backend
        * [**nostr-indexeddb**](./crates/nostr-indexeddb): IndexedDB storage backend
    * [**nostr-relay-pool**](./crates/nostr-relay-pool): Nostr Relay Pool
    * [**nostr-sdk**](./crates/nostr-sdk): High level client library
    * [**nwc**](./crates/nwc): Nostr Wallet Connect (NWC) client
* Binaries (tools):
    * [**nostr-cli**](./crates/nostr-cli): Nostr CLI

### Embedded

**nostr** crate can be used in [`no_std`](https://docs.rust-embedded.org/book/intro/no-std.html) environments. 
Check the example in the [`embedded/`](./crates/nostr/examples/embedded) directory.

## Book

Learn more about `rust-nostr` at <https://rust-nostr.org>.

## Available packages

* **nostr**:
    * Rust: https://crates.io/crates/nostr
* **nostr-sdk** (re-export everything from `nostr` library):
    * Rust: https://crates.io/crates/nostr-sdk
    * Python: https://pypi.org/project/nostr-sdk
    * Kotlin: 
      * Android: https://central.sonatype.com/artifact/org.rust-nostr/nostr-sdk
      * JVM: https://central.sonatype.com/artifact/org.rust-nostr/nostr-sdk-jvm
    * Swift: https://github.com/rust-nostr/nostr-sdk-swift
    * JavaScript: https://www.npmjs.com/package/@rust-nostr/nostr-sdk
    * Flutter: https://github.com/rust-nostr/nostr-sdk-flutter

## State

**These libraries are in ALPHA state**, things that are implemented generally work but the API will change in breaking ways.

## Donations

`rust-nostr` is free and open-source. This means we do not earn any revenue by selling it. Instead, we rely on your financial support. If you actively use any of the `rust-nostr` libs/software/services, then please [donate](https://rust-nostr.org/donate).

## License

This project is distributed under the MIT software license - see the [LICENSE](LICENSE) file for details
