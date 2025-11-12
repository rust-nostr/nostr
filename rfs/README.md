# Remote File Storage implementations

This workspace folder hosts the crates that implement [NIP-96](https://github.com/nostr-protocol/nips/blob/master/96.md) compatible uploads and the experimental [Blossom](https://github.com/hzrd149/blossom) protocol support.

- [`nostr-http-file-storage`](./nostr-http-file-storage) – async HTTP client that knows how to discover `nip96.json`, sign upload requests, and return permanent download URLs.
- [`nostr-blossom`](./nostr-blossom) – builder blocks for the Blossom protocol (basic client support today).

Each crate can be used stand-alone; nothing in here is pulled in automatically by `nostr` or `nostr-sdk`. Enable whichever fits your application via Cargo features.
