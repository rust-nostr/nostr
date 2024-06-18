# Timestamp

As a part of the Nostr protocol, events include a `created_at` field which contains the UNIX timestamp (in seconds) when the event was created. 
Note that this field can be useful in conjunction with the `since` and `until` properties of filters (see [Filters section](05_01_01-filters.md)) to help clients surface recent/relevant content.

The [Timestamp](https://docs.rs/nostr/latest/nostr/types/time/struct.Timestamp.html) struct is responsible for handling the creation and/or parsing of timestamp objects. 
This section also covers the `custom_created_at()` method from the `EventBuilder` struct and the `expiration` tag used in functions like `gift_wrap()`. 

## Creating, Parsing and Presenting Timestamps

<custom-tabs category="lang">

<div slot="title">Rust</div>
<section>

TODO

</section>

<div slot="title">Python</div>
<section>

The `Timestamp` class is used to instantiate a timestamp object and the `now()` method can be used to populate this with the current UNIX timestamp. 
The `to_human_datetime()` and `as_secs()` methods can be used to present the timestamp data as a human-readable string or, 
UNIX integer timestamp in seconds, respectively.

```python,ignore
{{#include ../../snippets/nostr/python/src/timestamps.py:timestamp-now}}
```

To parse timestamps from integer values the `from_secs()` method can be used. 

```python,ignore
{{#include ../../snippets/nostr/python/src/timestamps.py:timestamp-parse}}
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

## Created_at and Expiration

<custom-tabs category="lang">

<div slot="title">Rust</div>
<section>

TODO

</section>

<div slot="title">Python</div>
<section>

When building `Event` objects it is possible to utilize the `custom_created_at()` method in conjunction with an instance 
of the `Timestamp` class to manually set the created_at field for events. When parsing events, the `create_at()` method is used to 
extract this timestamp information.

```python,ignore
{{#include ../../snippets/nostr/python/src/timestamps.py:timestamp-created}}
```

To create expiration tags for inclusion within events the `Tag` class is used along with the `expiration()` method.

```python,ignore
{{#include ../../snippets/nostr/python/src/timestamps.py:timestamp-tag}}
```

This example shows how the expiration tag can be set directly during the calling of the `gift_wrap()` function. 

```python,ignore
{{#include ../../snippets/nostr/python/src/timestamps.py:timestamp-expiration}}
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