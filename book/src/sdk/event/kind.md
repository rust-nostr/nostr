## Kind

As a core component of nostr objects, kinds are used to signal to clients how to parse the data contained within an event. 
A `kind` is represented by an integer between `0` and `65535` the most well known of which is the Kind `1`, 
or `text note` which contains plaintext data to be displayed. 
Other commonly used kinds include kind `0` (user metadata) and Kind `3` (following/contact lists). 
For more details and to see the full range of proposed/adopted Kinds please refer to the [Nostr NIPs documentation](https://github.com/nostr-protocol/nips/tree/master?tab=readme-ov-file#event-Kinds).

### Kind by Integer and Enum

<custom-tabs category="lang">

<div slot="title">Rust</div>
<section>

TODO

</section>

<div slot="title">Python</div>
<section>

Working with kinds is facilitated by the `Kind` and `KindEnum` classes. 
If you are familiar already with the specific integer value for a given Kind it is as simple as calling and instance of the class `Kind()` and passing the specific number for the Kind you wish to create.

In the example below we've used the common `0`/`1`/`3` Kinds (user metadata, text note and following list, respectively) as an illustration of this. 
Once we've created the `Kind` object we can use the `as_enum()` method to present the Kind object as an easy to read `KindEnum` object.

```python,ignore
{{#include ../../../snippets/python/src/event/kind.py:kind-int}}
```

Alternatively, if you are less familiar with the specific integer values for a Kind we can use the individual Kind classes, in conjunction with the `KindEnum` class, to generate the objects. 
Below we see the `TEXT_NOTE()`, `METADATA()` and `CONTACT_LIST()` enums being passed to an instance of the `Kind` class via the `from_enum()` method.

In order to present these as their integer values we can use the `as_u16()` or `as_u64()` methods.

```python,ignore
{{#include ../../../snippets/python/src/event/kind.py:kind-enum}}
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

<div slot="title">Flutter</div>
<section>

TODO

</section>
</custom-tabs>

### Events and Kinds

<custom-tabs category="lang">

<div slot="title">Rust</div>
<section>

TODO

</section>

<div slot="title">Python</div>
<section>

Although it's possible to construct `EventBuilder` objects by passing the `Kind` class as the first argument (see [Event](index.md) section for examples), 
one of the simplest ways of constructing `Event` objects is by using the purpose built methods available to the `EventBuilder` class. 
For example, the `text_note()` method can be used to quickly and efficiently create Kind 1 events, the `metadata()` and `contact_list()` methods can be used in much the same way.

```python,ignore
{{#include ../../../snippets/python/src/event/kind.py:kind-methods}}
```

Occasionally you may want more generic usage of kinds, like if you wanted to create your own custom (or experimental) event type, 
or if you want to leverage one of the commonly defined event types (i.e. replaceable, ephemeral, regular etc.). 
To do this we can use the `Kind` class along with the `from_enum()` method much as we did in previous examples, 
but we can leverage enums representing these types of events e.g. `CUSTOM()` or `REPLACEABLE()` and pass them the specific Kind integer for the new type of event we're creating. 

A good example of this may be events termed as "Parameterized Replaceable Lists". 
In the [Nostr NIP-01 documentation](https://github.com/nostr-protocol/nips/blob/master/01.md) we see a recommended range for these lists as `30000 <= n < 40000`, 
however at the time of writing, only kinds `30000`, `30002`, `30003`, `30004`, `30005`, `30015`, `30030` and `30063` are currently well defined. 
Therefore, if we wanted to extend this to say create a new list event of our favourite memes, Kind `30420`, 
then we could do this using the `PARAMETERIZED_REPLACEABLE(30420)` enum to define the type of event as in the example below.

```python,ignore
{{#include ../../../snippets/python/src/event/kind.py:kind-representations}}
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

<div slot="title">Flutter</div>
<section>

TODO

</section>
</custom-tabs>
