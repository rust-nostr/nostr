# Nostr

## Structure

- [bindings](./bindings/):
    - [nostr-sdk-ffi](./bindings/nostr-sdk-ffi/): UniFFI bindings of the nostr-sdk crate
- [crates](./crates/):
    - [nostr-sdk](./crates/nostr-sdk/): High level client library.
    - [nostr-sdk-base](./crates/nostr-sdk-base/): No (network) IO client state machine that can be used to embed a Nostr client in your project or build a full fledged network enabled client lib on top of it.

## Minimum Supported Rust Version (MSRV)

These crates are built with the Rust language version 2021 and require a minimum compiler version of `1.64`

## Bindings

**nostr-sdk** crate can be embedded inside other environments, like Swift and Kotlin. 
Please, explore the [`bindings/`](./bindings/) directory to learn more.

## State

**This library is in an ALPHA state**, things that are implemented generally work but the API will change in breaking ways.

## License

This project is distributed under the MIT software license - see the [LICENSE](LICENSE) file for details