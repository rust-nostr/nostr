# Keys

## Generate new random keys

To generate a new key pair use the `generate()` method:

<custom-tabs category="lang">

<div slot="title">Rust</div>
<section>

```rust,ignore
{{#include ../../snippets/nostr/rust/src/keys.rs:generate}}
```

</section>

<div slot="title">Python</div>
<section>

```python,ignore
{{#include ../../snippets/nostr/python/src/keys.py:generate}}
```

</section>

<div slot="title">JavaScript</div>
<section>

```javascript,ignore
{{#include ../../snippets/nostr/js/src/keys.js:generate}}
```

</section>

<div slot="title">Kotlin</div>
<section>

TODO

</section>

<div slot="title">Swift</div>
<section>

```swift
{{#include ../../snippets/nostr/swift/NostrSnippets/Sources/Keys.swift}}
```

</section>
</custom-tabs>

## Restore from hex and/or bech32 secret key

<custom-tabs category="lang">

<div slot="title">Rust</div>
<section>

```rust,ignore
{{#include ../../snippets/nostr/rust/src/keys.rs:restore}}
```

</section>

<div slot="title">Python</div>
<section>

```python,ignore
{{#include ../../snippets/nostr/python/src/keys.py:restore}}
```

</section>

<div slot="title">JavaScript</div>
<section>

```javascript,ignore
{{#include ../../snippets/nostr/js/src/keys.js:restore}}
```

</section>

<div slot="title">Kotlin</div>
<section>

TODO

</section>

<div slot="title">Swift</div>
<section>

TODO

</section>
</custom-tabs>

## Generate vanity keys

<custom-tabs category="lang">

<div slot="title">Rust</div>
<section>

```rust,ignore
{{#include ../../snippets/nostr/rust/src/keys.rs:vanity}}
```

</section>

<div slot="title">Python</div>
<section>

```python,ignore
{{#include ../../snippets/nostr/python/src/keys.py:vanity}}
```

</section>

<div slot="title">JavaScript</div>
<section>

```javascript,ignore
{{#include ../../snippets/nostr/js/src/keys.js:vanity}}
```

</section>

<div slot="title">Kotlin</div>
<section>

TODO

</section>

<div slot="title">Swift</div>
<section>

```swift
{{#include ../../snippets/nostr/swift/NostrSnippets/Sources/Vanity.swift}}
```

</section>
</custom-tabs>