# Nostr

## Structure

- [bindings](./bindings/):
    - [nostr-ffi](./bindings/nostr-ffi/): UniFFI bindings of the matrix crate
- [nostr](./nostr/): Implementation of nostr protocol
- [nostr-sdk](./nostr-sdk/): High level client library build on top of nostr crate

## Bindings

**nostr** and **nostr-sdk** crates can be embedded inside other environments, like Swift and Kotlin. 
Please, explore the [`bindings/`](./bindings/) directory to learn more.

## State

**This library is in an ALPHA state**, things that are implemented generally work but the API will change in breaking ways.

## License

This project is distributed under the MIT software license - see the [LICENSE](LICENSE) file for details