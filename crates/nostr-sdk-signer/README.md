# Nostr SDK Signer

## Crate Feature Flags

The following crate feature flags are available:

| Feature             | Default | Description                                                                                 |
| ------------------- | :-----: | ------------------------------------------------------------------------------------------- |
| `blocking`          |   No    | Needed to use `NIP-05` and `NIP-11` features in not async/await context                     |
| `nip04`             |   Yes   | Enable NIP-04: Encrypted Direct Message                                                     |
| `nip07`             |   Yes   | Enable NIP-07: `window.nostr` capability for web browsers (**available only for `wasm32`!**)|
| `nip44`             |   Yes   | Enable NIP-44: Encrypted Payloads (Versioned)                                               |
| `nip46`             |   Yes   | Enable NIP-46: Nostr Connect                                                                |

## State

**This library is in an ALPHA state**, things that are implemented generally work but the API will change in breaking ways.

## Donations

`rust-nostr` is free and open-source. This means we do not earn any revenue by selling it. Instead, we rely on your financial support. If you actively use any of the `rust-nostr` libs/software/services, then please [donate](https://rust-nostr.org/donate).

## License

This project is distributed under the MIT software license - see the [LICENSE](../../LICENSE) file for details