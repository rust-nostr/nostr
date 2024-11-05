## Installing the library

<custom-tabs category="lang">

<div slot="title">Rust</div>
<section>

Add the `nostr` dependency in your `Cargo.toml` file:

```toml,ignore
[dependencies]
nostr = "0.36"
```

Alternatively, you can add it directly from `git` source:

```toml,ignore
[dependencies]
nostr = { git = "https://github.com/rust-nostr/nostr", tag = "v0.36.0" }
```

```admonish info
To use a specific commit, use `rev` instead of `tag`.
```

Import the library in your code:

```rust,ignore
use nostr::prelude::*;
```

</section>

<div slot="title">Python</div>
<section>

Check [nostr-sdk](../nostr-sdk/installation.md) installation docs.

</section>

<div slot="title">JavaScript</div>
<section>

Check [nostr-sdk](../nostr-sdk/installation.md) installation docs.

</section>

<div slot="title">Kotlin</div>
<section>

Check [nostr-sdk](../nostr-sdk/installation.md) installation docs.

</section>

<div slot="title">Swift</div>
<section>

Check [nostr-sdk](../nostr-sdk/installation.md) installation docs.

</section>

<div slot="title">Flutter</div>
<section>

Check [nostr-sdk](../nostr-sdk/installation.md) installation docs.

</section>
</custom-tabs>
