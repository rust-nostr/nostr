# Features

!!! note
    This page is related **only** to the `rust` library. 

Available features:

| Feature             | Default | Description                                                                              |
| ------------------- | :-----: | ---------------------------------------------------------------------------------------- |
| `std`               |   Yes   | Enable `std` library                                                                     |
| `alloc`             |   No    | Needed to use this library in `no_std` context                                           |
| `blocking`          |   No    | Needed to use `NIP-05` and `NIP-11` features in not async/await context                  |
| `all-nips`          |   Yes   | Enable all NIPs                                                                          |
| `nip03`             |   No    | Enable NIP-03: OpenTimestamps Attestations for Events                                    |
| `nip04`             |   Yes   | Enable NIP-04: Encrypted Direct Message                                                  |
| `nip05`             |   Yes   | Enable NIP-05: Mapping Nostr keys to DNS-based internet identifiers                      |
| `nip06`             |   Yes   | Enable NIP-06: Basic key derivation from mnemonic seed phrase                            |
| `nip11`             |   Yes   | Enable NIP-11: Relay Information Document                                                |
| `nip44`             |   No    | Enable NIP-44: Encrypted Payloads (Versioned) - EXPERIMENTAL                             |
| `nip46`             |   Yes   | Enable NIP-46: Nostr Connect                                                             |
| `nip47`             |   Yes   | Enable NIP-47: Nostr Wallet Connect                                                      |