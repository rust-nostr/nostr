## Event Building

A convenient way to create events is by using the `EventBuilder`. 
It allows to build `standard` and/or `custom` events.

### Construct the event

Standard events can be composed by using the dedicated constructors.
In the below example we are going to build a text note:

<custom-tabs category="lang">

<div slot="title">Rust</div>
<section>

```rust,ignore
{{#include ../../../snippets/rust/src/event/builder.rs:standard}}
```

</section>

<div slot="title">Python</div>
<section>

```python,ignore
{{#include ../../../snippets/python/src/event/builder.py:standard}}
```

</section>

<div slot="title">JavaScript</div>
<section>

```typescript,ignore
{{#include ../../../snippets/js/src/event/builder.ts:standard}}
```

</section>

<div slot="title">Kotlin</div>
<section>

```kotlin,ignore
{{#include ../../../snippets/kotlin/shared/src/main/kotlin/rust/nostr/snippets/event/Builder.kt:standard}}
```

</section>

<div slot="title">Swift</div>
<section>

```swift,ignore
{{#include ../../../snippets/swift/NostrSnippets/Sources/Event/Builder.swift:standard}}
```

</section>

<div slot="title">Flutter</div>
<section>

```dart,ignore
{{#include ../../../snippets/flutter/lib/event/builder.dart:standard}}
```

</section>
</custom-tabs>

You can also customize the builder, for example, by specifying a POW difficulty, 
setting a fixed timestamp or adding more tags:

<custom-tabs category="lang">

<div slot="title">Rust</div>
<section>

```rust,ignore
{{#include ../../../snippets/rust/src/event/builder.rs:std-custom}}
```

</section>

<div slot="title">Python</div>
<section>

```python,ignore
{{#include ../../../snippets/python/src/event/builder.py:std-custom}}
```

</section>

<div slot="title">JavaScript</div>
<section>

```typescript,ignore
{{#include ../../../snippets/js/src/event/builder.ts:std-custom}}
```

</section>

<div slot="title">Kotlin</div>
<section>

```kotlin,ignore
{{#include ../../../snippets/kotlin/shared/src/main/kotlin/rust/nostr/snippets/event/Builder.kt:std-custom}}
```

</section>

<div slot="title">Swift</div>
<section>

```swift,ignore
{{#include ../../../snippets/swift/NostrSnippets/Sources/Event/Builder.swift:std-custom}}
```

</section>

<div slot="title">Flutter</div>
<section>

```dart,ignore
{{#include ../../../snippets/flutter/lib/event/builder.dart:std-custom}}
```

</section>
</custom-tabs>

If you need to create a non-standard event, you can use the default `EventBuilder` constructor:

<custom-tabs category="lang">

<div slot="title">Rust</div>
<section>

```rust,ignore
{{#include ../../../snippets/rust/src/event/builder.rs:custom}}
```

</section>

<div slot="title">Python</div>
<section>

```python,ignore
{{#include ../../../snippets/python/src/event/builder.py:custom}}
```

</section>

<div slot="title">JavaScript</div>
<section>

```typescript,ignore
{{#include ../../../snippets/js/src/event/builder.ts:custom}}
```

</section>

<div slot="title">Kotlin</div>
<section>

```kotlin,ignore
{{#include ../../../snippets/kotlin/shared/src/main/kotlin/rust/nostr/snippets/event/Builder.kt:custom}}
```

</section>

<div slot="title">Swift</div>
<section>

```swift,ignore
{{#include ../../../snippets/swift/NostrSnippets/Sources/Event/Builder.swift:custom}}
```

</section>

<div slot="title">Flutter</div>
<section>

```dart,ignore
{{#include ../../../snippets/flutter/lib/event/builder.dart:custom}}
```

</section>
</custom-tabs>

### Build and sign the event

After the `EventBuilder` construction, you can finally build and sign the event:

<custom-tabs category="lang">

<div slot="title">Rust</div>
<section>

```rust,ignore
{{#include ../../../snippets/rust/src/event/builder.rs:sign}}
```

</section>

<div slot="title">Python</div>
<section>

```python,ignore
{{#include ../../../snippets/python/src/event/builder.py:sign}}
```

</section>

<div slot="title">JavaScript</div>
<section>

```typescript,ignore
{{#include ../../../snippets/js/src/event/builder.ts:sign}}
```

</section>

<div slot="title">Kotlin</div>
<section>

```kotlin,ignore
{{#include ../../../snippets/kotlin/shared/src/main/kotlin/rust/nostr/snippets/event/Builder.kt:sign}}
```

</section>

<div slot="title">Swift</div>
<section>

```swift,ignore
{{#include ../../../snippets/swift/NostrSnippets/Sources/Event/Builder.swift:sign}}
```

</section>

<div slot="title">Flutter</div>
<section>

```dart,ignore
{{#include ../../../snippets/flutter/lib/event/builder.dart:sign}}
```

</section>
</custom-tabs>

### Full example

<custom-tabs category="lang">

<div slot="title">Rust</div>
<section>

```rust,ignore
{{#include ../../../snippets/rust/src/event/builder.rs:full}}
```

</section>

<div slot="title">Python</div>
<section>

```python,ignore
{{#include ../../../snippets/python/src/event/builder.py:full}}
```

</section>

<div slot="title">JavaScript</div>
<section>

```typescript,ignore
{{#include ../../../snippets/js/src/event/builder.ts:full}}
```

</section>

<div slot="title">Kotlin</div>
<section>

```kotlin,ignore
{{#include ../../../snippets/kotlin/shared/src/main/kotlin/rust/nostr/snippets/event/Builder.kt:full}}
```

</section>

<div slot="title">Swift</div>
<section>

```swift,ignore
{{#include ../../../snippets/swift/NostrSnippets/Sources/Event/Builder.swift:full}}
```

</section>

<div slot="title">Flutter</div>
<section>

```dart,ignore
{{#include ../../../snippets/flutter/lib/event/builder.dart:full}}
```

</section>
</custom-tabs>
