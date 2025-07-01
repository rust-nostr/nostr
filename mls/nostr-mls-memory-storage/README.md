# Nostr MLS Memory Storage

Memory-based storage implementation for [Nostr MLS](../nostr-mls). This crate provides a storage backend that implements the `NostrMlsStorageProvider` trait from the [nostr-mls-storage](../nostr-mls-storage) crate.

## Features

- Uses an LRU (Least Recently Used) caching mechanism to store data in memory
- Provides both read and write operations that are thread-safe through `parking_lot::RwLock`
- Configurable cache size (default: 1000 items)
- Non-persistent storage that is cleared when the application terminates

## Performance

This implementation uses `parking_lot::RwLock` instead of the standard library's `std::sync::RwLock` for improved performance. The `parking_lot` implementation offers several advantages:

- Smaller memory footprint
- Faster lock acquisition and release
- No poisoning on panic
- More efficient read-heavy workloads, which is ideal for this caching implementation
- Consistent behavior across different platforms

## Example Usage

```rust,ignore
use nostr_mls_memory_storage::NostrMlsMemoryStorage;
use nostr_mls_storage::NostrMlsStorageProvider;

// Create a new memory storage instance
let storage = NostrMlsMemoryStorage::default();

// Or create with a custom cache size
let custom_storage = NostrMlsMemoryStorage::with_cache_size(100);
```

For more advanced usage examples, see the tests in the source code.

## Changelog

All notable changes to this library are documented in the [CHANGELOG.md](CHANGELOG.md).

## State

**This library is in an ALPHA state**, things that are implemented generally work but the API will change in breaking ways.

## Donations

`rust-nostr` is free and open-source. This means we do not earn any revenue by selling it. Instead, we rely on your financial support. If you actively use any of the `rust-nostr` libs/software/services, then please [donate](https://rust-nostr.org/donate).

## License

This project is distributed under the MIT software license - see the [LICENSE](../../LICENSE) file for details
