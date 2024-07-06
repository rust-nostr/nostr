# Event ID

An event id is defined per the [Nostr NIP-01 documentation](https://github.com/nostr-protocol/nips/blob/master/01.md) as the `32-bytes lowercase hex-encoded sha256 of the serialised event data`. It is fundamentally a unique identifier for an event generated from the hash of the content of a Nostr event object (excluding the signature).

The [EventID](https://docs.rs/nostr/latest/nostr/event/id/struct.EventId.html) struct is predominantly responsible for creation of, and working with event id objects. 


## Creation, Formatting and Parsing

<custom-tabs category="lang">

<div slot="title">Rust</div>
<section>

TODO

</section>

<div slot="title">Python</div>
<section>

The `EventID` class can be called in order to construct event ids, although this is not necessary when building `Event` objects as it will be done automatically at that time. 

Upon instantiation the following content are passed to the class instance to generate the event id; `public_key`, `created_at`, `kind`, `tags` and `content`. For more information about these individual objects please refer to the relevant sections; [Keys](03-keys.md), [Timestamp](04_03-timestamp.md), [Kind](04_02-kind.md) and [Tag](04_04-tag.md), respectively.

```python,ignore
{{#include ../../snippets/nostr/python/src/event/eventid.py:build-event-id}}
```

Once we have an event id object we are able to format and parse this using a few simple methods. To present as a hex, bech32, nostr uri or as bytes we need only call the relevant methods `to_hex()`, `to_bech32()`, `to_nostr_uri()` or `to_bytes()`. Similarly, we can parse these different representations of the event id by using the opposite 'from' methods; `from_hex()`, `from_bech32()`, `from_nostr_uri()`, or `from_bytes()`.

In the event that we want to generalise and simplify this process, across hex/bech32 or nostr uri formats, we can instead simply call `parse()` method and pass this the event id string matching one of these formats. 

For more information/examples on the formatting of Nostr objects please refer to [NIP-19](06-nip19.md) and [NIP-21](06-nip21.md).

```python,ignore
{{#include ../../snippets/nostr/python/src/event/eventid.py:format-parse-hex}}
```

```python,ignore
{{#include ../../snippets/nostr/python/src/event/eventid.py:format-parse-bech32}}
```

```python,ignore
{{#include ../../snippets/nostr/python/src/event/eventid.py:format-parse-nostr-uri}}
```

```python,ignore
{{#include ../../snippets/nostr/python/src/event/eventid.py:format-parse-bytes}}
```

</section>

<div slot="title">JavaScript</div>
<section>

TODO

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

## Access and Verify

<custom-tabs category="lang">

<div slot="title">Rust</div>
<section>

TODO

</section>

<div slot="title">Python</div>
<section>

In addition to directly creating/manipulating event id objects we can also easily access these directly from events, by calling the `id()` method on and instance of the `Event` class, or, verify that the event id (and signature) for an event is valid, by using the `verify()` method.  

```python,ignore
{{#include ../../snippets/nostr/python/src/event/eventid.py:access-verify}}
```

</section>

<div slot="title">JavaScript</div>
<section>

TODO

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