# Nostr SQLite database backend

SQLite storage backend for nostr apps.

## Crate Feature Flags

The following crate feature flags are available:

| Feature   | Default | Description         |
|-----------|:-------:|---------------------|
| `bundled` |   Yes   | Uses bundled SQLite |

## Supported NIPs

| Supported | NIP                                                                                   |
|:---------:|---------------------------------------------------------------------------------------|
|     ❌     | [40 - Expiration Timestamp](https://github.com/nostr-protocol/nips/blob/master/40.md) |
|     ✅     | [50 - Search Capability](https://github.com/nostr-protocol/nips/blob/master/50.md)    |
|    ✅*     | [62 - Request to Vanish](https://github.com/nostr-protocol/nips/blob/master/62.md)    |

*: `ALL_RELAYS` only

## State

**This library is in an ALPHA state**, things that are implemented generally work but the API will change in breaking ways.

## Donations

`rust-nostr` is free and open-source. This means we do not earn any revenue by selling it. Instead, we rely on your financial support. If you actively use any of the `rust-nostr` libs/software/services, then please [donate](https://rust-nostr.org/donate).

## License

This project is distributed under the MIT software license - see the [LICENSE](../../LICENSE) file for details
