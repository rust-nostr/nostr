## Quickstart

<custom-tabs category="lang">

<div slot="title">Rust</div>
<section>

Create a client and connect to some relays.

```rust,ignore
{{#include ../../snippets/nostr-sdk/rust/src/quickstart.rs:5:26}}
```

Add metadata for the keys in the existing client.

```rust,ignore
{{#include ../../snippets/nostr-sdk/rust/src/quickstart.rs:28:40}}
```

Create a filter and notify the relays of the subscription.

```rust,ignore
{{#include ../../snippets/nostr-sdk/rust/src/quickstart.rs:42:43}}
```

For more supported filters, view [the documentation](https://docs.rs/nostr-sdk/latest/nostr_sdk/struct.Filter.html).

Listen for notifications from the relays based on the subscribed filters and process them some way.

```rust, ignore
{{#include ../../snippets/nostr-sdk/rust/src/quickstart.rs:45:54}}
```

</section>
