# Nostr

## Project structure

The project is split up into several crates in the `crates/` directory:

* [**nostr**](./crates/nostr/): Rust implementation of Nostr protocol.
* [**nostr-sdk**](./crates/nostr-sdk/): High level client library.
* [**nostr-sdk-net**](./crates/nostr-sdk-net/): Network library for [**nostr-sdk**](./crates/nostr-sdk/)

### Bindings

**nostr** and **nostr-sdk** crates can be embedded inside other environments, like Swift, Kotlin, Python and JavaScript. 
Please, explore the [`bindings/`](./bindings/) directory to learn more.

#### Available packages

* **nostr**:
    * Kotlin: [`io.github.rust-nostr:nostr`](https://central.sonatype.com/artifact/io.github.rust-nostr/nostr/)
    * Swift: https://github.com/rust-nostr/nostr-swift
    * Python: https://pypi.org/project/nostr-protocol
    * JavaScript: https://www.npmjs.com/package/@rust-nostr/nostr
* **nostr-sdk**:
    * Kotlin: [`io.github.rust-nostr:nostr-sdk`](https://central.sonatype.com/artifact/io.github.rust-nostr/nostr-sdk/)
    * Swift: https://github.com/rust-nostr/nostr-sdk-swift
    * Python: https://pypi.org/project/nostr-sdk
    * JavaScript: TODO

## Minimum Supported Rust Version (MSRV)

These crates are built with the Rust language version 2021 and require a minimum compiler version of `1.64.0`

## State

**These libraries are in ALPHA state**, things that are implemented generally work but the API will change in breaking ways.

## License

This project is distributed under the MIT software license - see the [LICENSE](LICENSE) file for details

## Donations

⚡ Tips: <https://getalby.com/p/yuki>

⚡ Lightning Address: yuki@getalby.com