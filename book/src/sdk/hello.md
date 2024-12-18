## Hello, rust-nostr!

Now that you’ve installed the SDK, it’s time to write your first nostr program.

### Generate random keys and construct the client

The first step is to generate random keys needed for the client and construct the client instance.

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

```swift,ignore
{{#include ../../snippets/swift/NostrSnippets/Sources/Hello.swift:client}}
```

</section>

<div slot="title">Flutter</div>
<section>

```dart,ignore
{{#include ../../snippets/flutter/lib/hello.dart:client}}
```

</section>
</custom-tabs>

### Add some relays and connect

Next, add some relays to your client and connect to them.

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

```swift,ignore
{{#include ../../snippets/swift/NostrSnippets/Sources/Hello.swift:connect}}
```

</section>

<div slot="title">Flutter</div>
<section>

```dart,ignore
{{#include ../../snippets/flutter/lib/hello.dart:connect}}
```

</section>
</custom-tabs>


### Publish a text note

Now that the client is constructed and the relays are connected, 
build a text note with the [EventBuilder](event/builder.md) and publish it to relays.

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

```swift,ignore
{{#include ../../snippets/swift/NostrSnippets/Sources/Hello.swift:publish}}
```

</section>

<div slot="title">Flutter</div>
<section>

```dart,ignore
{{#include ../../snippets/flutter/lib/hello.dart:publish}}
```

</section>
</custom-tabs>

### Inspect the output

Published the event, you can inspect the output to ensure everything worked correctly.

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

```swift,ignore
{{#include ../../snippets/swift/NostrSnippets/Sources/Hello.swift:output}}
```

</section>

<div slot="title">Flutter</div>
<section>

```dart,ignore
{{#include ../../snippets/flutter/lib/hello.dart:output}}
```

</section>
</custom-tabs>

### Full example

Here’s the full example that includes all the steps from generating keys to inspecting the output.

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

```swift,ignore
{{#include ../../snippets/swift/NostrSnippets/Sources/Hello.swift:full}}
```

</section>

<div slot="title">Flutter</div>
<section>

```dart,ignore
{{#include ../../snippets/flutter/lib/hello.dart:full}}
```

</section>
</custom-tabs>
