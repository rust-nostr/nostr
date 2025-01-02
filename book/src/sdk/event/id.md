## Event ID

An event ID is defined per the [Nostr NIP-01 documentation](https://github.com/nostr-protocol/nips/blob/master/01.md) as the `32-bytes lowercase hex-encoded sha256 of the serialised event data`. 
It's fundamentally a unique identifier for an event generated from the hash of the content of a Nostr event object (excluding the signature).

### Creation, Formatting and Parsing

<custom-tabs category="lang">

<div slot="title">Rust</div>
<section>

TODO

</section>

<div slot="title">Python</div>
<section>

The `EventId` class can be called in order to construct event ids, although this is not necessary when building `Event` objects as it will be done automatically at that time. 

Upon instantiation the following content are passed to the class instance to generate the event ID: `public_key`, `created_at`, `kind`, `tags` and `content`.

```python,ignore
{{#include ../../../snippets/python/src/event/id.py:build-event-id}}
```

Once we have an event id object we are able to format and parse this using a few simple methods. 
To present as a hex, bech32, nostr uri or as bytes we need only call the relevant methods `to_hex()`, `to_bech32()`, `to_nostr_uri()` or `to_bytes()`. 

It is somewhat trivial to perform the reverse action given that this has been generalised across, hex/bech32 or nostr uri formats. This is achived by calling the `parse()` method and passing this the event id string matching one of these formats. The exception to this rule is for bytes where the `from_bytes()` method is to be used.

For more information/examples on the formatting of Nostr objects please refer to [NIP-19](../nips/19.md) and [NIP-21](../nips/21.md).

```python,ignore
{{#include ../../../snippets/python/src/event/id.py:format-parse-hex}}
```

```python,ignore
{{#include ../../../snippets/python/src/event/id.py:format-parse-bech32}}
```

```python,ignore
{{#include ../../../snippets/python/src/event/id.py:format-parse-nostr-uri}}
```

```python,ignore
{{#include ../../../snippets/python/src/event/id.py:format-parse-bytes}}
```

</section>

<div slot="title">JavaScript</div>
<section>

The `EventId` class can be called in order to construct event ids, although this is not necessary when building `Event` objects as it will be done automatically at that time. 

Upon instantiation the following content are passed to the class instance to generate the event ID: `publicKey`, `createdAt`, `kind`, `tags` and `content`.

```typescript,ignore
{{#include ../../../snippets/js/src/event/id.ts:build-event-id}}
```

Once we have an event id object we are able to format and parse this using a few simple methods. 
To present as a hex, bech32, nostr uri or as bytes we need only call the relevant methods `toHex()`, `toBech32()`, `toNostrUri()` or `asBytes()`. 

It is somewhat trivial to perform the reverse action given that this has been generalised across, hex/bech32 or nostr uri formats. This is achived by calling the `parse()` method and passing this the event id string matching one of these formats. The exception to this rule is for bytes where the `fromBytes()` method is to be used.

For more information/examples on the formatting of Nostr objects please refer to [NIP-19](../nips/19.md) and [NIP-21](../nips/21.md).

```typescript,ignore
{{#include ../../../snippets/js/src/event/id.ts:format-parse-hex}}
```

```typescript,ignore
{{#include ../../../snippets/js/src/event/id.ts:format-parse-bech32}}
```

```typescript,ignore
{{#include ../../../snippets/js/src/event/id.ts:format-parse-nostr-uri}}
```

```typescript,ignore
{{#include ../../../snippets/js/src/event/id.ts:format-parse-bytes}}
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

<div slot="title">Flutter</div>
<section>

TODO

</section>
</custom-tabs>

### Access and Verify

<custom-tabs category="lang">

<div slot="title">Rust</div>
<section>

TODO

</section>

<div slot="title">Python</div>
<section>

In addition to directly creating/manipulating event ID objects we can also easily access these directly from events, by calling the `id()` method on and instance of the `Event` class, or, verify that the event id (and signature) for an event is valid, by using the `verify()` method.  

```python,ignore
{{#include ../../../snippets/python/src/event/id.py:access-verify}}
```

</section>

<div slot="title">JavaScript</div>
<section>

In addition to directly creating/manipulating event ID objects we can also easily access these directly from events, by calling the `id()` method on and instance of the `Event` class, or, verify that the event id (and signature) for an event is valid, by using `verify()` method for both Signature & ID or the `verifyId()` method for the ID alone.  

```typescript,ignore
{{#include ../../../snippets/js/src/event/id.ts:access-verify}}
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

<div slot="title">Flutter</div>
<section>

TODO

</section>
</custom-tabs>
