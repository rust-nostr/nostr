## Hello, rust-nostr!

Now that you’ve installed the SDK, it’s time to write your first nostr program.

### Generate random keys and construct the client

<custom-tabs category="lang">
<div slot="title">Rust</div>
<section>

```rust,ignore
{{#include ../../snippets/rust/src/hello.rs:client}}
```

</section>

<div slot="title">Python</div>
<section>

```python,ignore
{{#include ../../snippets/python/src/hello.py:client}}
```

</section>

<div slot="title">JavaScript</div>
<section>

```typescript,ignore
{{#include ../../snippets/js/src/hello.ts:client}}
```

</section>

<div slot="title">Kotlin</div>
<section>

```kotlin,ignore
{{#include ../../snippets/kotlin/shared/src/main/kotlin/rust/nostr/snippets/Hello.kt:client}}
```

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

### Add some relays and connect

<custom-tabs category="lang">
<div slot="title">Rust</div>
<section>

```rust,ignore
{{#include ../../snippets/rust/src/hello.rs:connect}}
```

</section>

<div slot="title">Python</div>
<section>

```python,ignore
{{#include ../../snippets/python/src/hello.py:connect}}
```

</section>

<div slot="title">JavaScript</div>
<section>

```typescript,ignore
{{#include ../../snippets/js/src/hello.ts:connect}}
```

</section>

<div slot="title">Kotlin</div>
<section>

```kotlin,ignore
{{#include ../../snippets/kotlin/shared/src/main/kotlin/rust/nostr/snippets/Hello.kt:connect}}
```

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


### Publish a text note

<custom-tabs category="lang">
<div slot="title">Rust</div>
<section>

```rust,ignore
{{#include ../../snippets/rust/src/hello.rs:publish}}
```

</section>

<div slot="title">Python</div>
<section>

```python,ignore
{{#include ../../snippets/python/src/hello.py:publish}}
```

</section>

<div slot="title">JavaScript</div>
<section>

```typescript,ignore
{{#include ../../snippets/js/src/hello.ts:publish}}
```

</section>

<div slot="title">Kotlin</div>
<section>

```kotlin,ignore
{{#include ../../snippets/kotlin/shared/src/main/kotlin/rust/nostr/snippets/Hello.kt:publish}}
```

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

### Inspect the output

<custom-tabs category="lang">
<div slot="title">Rust</div>
<section>

```rust,ignore
{{#include ../../snippets/rust/src/hello.rs:output}}
```

</section>

<div slot="title">Python</div>
<section>

```python,ignore
{{#include ../../snippets/python/src/hello.py:output}}
```

</section>

<div slot="title">JavaScript</div>
<section>

```typescript,ignore
{{#include ../../snippets/js/src/hello.ts:output}}
```

</section>

<div slot="title">Kotlin</div>
<section>

```kotlin,ignore
{{#include ../../snippets/kotlin/shared/src/main/kotlin/rust/nostr/snippets/Hello.kt:output}}
```

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

### Full example

<custom-tabs category="lang">
<div slot="title">Rust</div>
<section>

```rust,ignore
{{#include ../../snippets/rust/src/hello.rs:full}}
```

</section>

<div slot="title">Python</div>
<section>

```python,ignore
{{#include ../../snippets/python/src/hello.py:full}}
```

</section>

<div slot="title">JavaScript</div>
<section>

```typescript,ignore
{{#include ../../snippets/js/src/hello.ts:full}}
```

</section>

<div slot="title">Kotlin</div>
<section>

```kotlin,ignore
{{#include ../../snippets/kotlin/shared/src/main/kotlin/rust/nostr/snippets/Hello.kt:full}}
```

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
