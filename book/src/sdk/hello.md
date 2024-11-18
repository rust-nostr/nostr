## Hello, rust-nostr!

Now that you’ve installed the SDK, it’s time to write your first nostr program.

<custom-tabs category="lang">

<div slot="title">Rust</div>
<section>

### Generate random keys

```rust,ignore
{{#include ../../snippets/rust/src/hello.rs:keys}}
```

### Create a client

```rust,ignore
{{#include ../../snippets/rust/src/hello.rs:client}}
```

### Add some relays and connect

```rust,ignore
{{#include ../../snippets/rust/src/hello.rs:connect}}
```

### Publish a text note

```rust,ignore
{{#include ../../snippets/rust/src/hello.rs:publish}}
```

### Full example

```rust,ignore
{{#include ../../snippets/rust/src/hello.rs:all}}
```

</section>

<div slot="title">Python</div>
<section>

Docs aren't ready yet, please check the examples at <https://github.com/rust-nostr/nostr/tree/master/bindings/nostr-sdk-ffi/python/examples>.

</section>

<div slot="title">JavaScript</div>
<section>

Docs aren't ready yet, please check the examples at <https://github.com/rust-nostr/nostr/tree/master/bindings/nostr-sdk-js/examples>.

</section>

<div slot="title">Kotlin</div>
<section>

TODO

</section>

<div slot="title">Swift</div>
<section>

TODO

</section>

<div slot="title">Flutter</div>
<section>

TODO

</section>
</custom-tabs>
