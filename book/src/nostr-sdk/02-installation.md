# Installing the library

Add the `nostr-sdk` dependency in your `Cargo.toml` file:

```toml
[dependencies]
nostr-sdk = "0.24"
```

Alternatively, you can add it directly from `git` source:

```toml
[dependencies]
nostr-sdk = { git = "https://github.com/rust-nostr/nostr", tag = "vX.X.X" }
```

```admonish note
To use a specific commit, use `rev` instead of `tag`.
```
