# Relay Message

The backbone of the Nostr network is built on relays rather than application specific centralized databases. Clients use WebSockets as a means to connect to relays and pass relevant data back and forth around the network. In accordance with the protocol base specification ([NIP-01](https://github.com/nostr-protocol/nips/blob/master/01.md#from-relay-to-client-sending-events-and-notices)) there are 5 main types of messages which relays construct as JSON arrays. This section is concerned with the construction of these message objects using the [Relay Message Module](https://docs.rs/nostr/latest/nostr/message/relay/index.html). 

For a more detailed explanation regarding the rules and handling of relay message objects please refer to the Nostr protocol documentation linked above. 

## Serialize/deserialize to/from JSON

<custom-tabs category="lang">

<div slot="title">Rust</div>
<section>

```rust,ignore
{{#include ../../snippets/nostr/rust/src/messages/relay.rs}}
```

</section>

<div slot="title">Python</div>
<section>

The `RelayMessage` class easily handles the construction of the 5 main message types `EVENT`, `OK`, `EOSE` (end of stored events), `CLOSED` and `NOTICE`. In the examples below we can utilize the relevant class methods `event()`, `ok()`, `eose()`, `closed()` and `notice()`, respectively, to create the relay message objects.

Once we have the `RelayMessage` objects we can use the `as_enum()` or `as_json()` methods to present their content. Note that when using `as_enum()` we unlock some additional methods associated with the `RelayMessageEnum` class. These allow for logical tests to be performed to establish the type of message object being assessed (for example, `is_ok()` will return a bool result assessing if the object represents an `OK` message type).  

```python,ignore
{{#include ../../snippets/nostr/python/src/messages/relay.py:event-message}}
```

```python,ignore
{{#include ../../snippets/nostr/python/src/messages/relay.py:ok-message}}
```

```python,ignore
{{#include ../../snippets/nostr/python/src/messages/relay.py:eose-message}}
```

```python,ignore
{{#include ../../snippets/nostr/python/src/messages/relay.py:closed-message}}
```

```python,ignore
{{#include ../../snippets/nostr/python/src/messages/relay.py:notice-message}}
```

When presented with a relay message object as either a JSON or an instance of the `RelayMessageEnum` class we can parse these data using the `from_json()` or `from_enum()` methods, respectively.

```python,ignore
{{#include ../../snippets/nostr/python/src/messages/relay.py:parse-message}}
```

</section>

<div slot="title">JavaScript</div>
<section>


The `RelayMessage` class easily handles the construction of the 5 main message types `EVENT`, `OK`, `EOSE` (end of stored events), `CLOSED` and `NOTICE`. In the examples below we can utilize the relevant class methods `event()`, `ok()`, `eose()`, `closed()` and `notice()`, respectively, to create the relay message objects.

Once we have the `RelayMessage` objects we can use the `asJson()` method to present their content. 

```javascript,ignore
{{#include ../../snippets/nostr/js/src/messages/relay.js:event-message}}
```

```javascript,ignore
{{#include ../../snippets/nostr/js/src/messages/relay.py:ok-message}}
```

```javascript,ignore
{{#include ../../snippets/nostr/js/src/messages/relay.js:eose-message}}
```

```javascript,ignore
{{#include ../../snippets/nostr/js/src/messages/relay.js:closed-message}}
```

```javascript,ignore
{{#include ../../snippets/nostr/js/src/messages/relay.js:notice-message}}
```

When presented with a relay message object as either a JSON we can parse these data using the `fromJson()` method.

```javascript,ignore
{{#include ../../snippets/nostr/js/src/messages/relay.js:parse-message}}
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

## Authorization and Count Messages

<custom-tabs category="lang">

<div slot="title">Rust</div>
<section>

TODO

</section>

<div slot="title">Python</div>
<section>

As an extension of the relay messaging section of the protocol [NIP-42](https://github.com/nostr-protocol/nips/blob/master/42.md) and [NIP-45](https://github.com/nostr-protocol/nips/blob/master/45.md) introduce two new messaging types `AUTH` and `COUNT`.

The `AUTH` type is designed to facilitate a method by which clients can authenticate with a given relay. Whereas the `COUNT` type offers a method for relays to provide simple counts of events to clients (upon request). These are constructed in much the same way as the earlier message examples, by using the `RelayMessage` class in conjunction with the relevant methods `auth()` and `count()`. As before the `as_enum()` method can be used to unlock logical test methods (e.g., `is_auth()`) associated with these message objects.

```python,ignore
{{#include ../../snippets/nostr/python/src/messages/relay.py:auth-message}}
```

```python,ignore
{{#include ../../snippets/nostr/python/src/messages/relay.py:count-message}}
```

</section>

<div slot="title">JavaScript</div>
<section>

As an extension of the relay messaging section of the protocol [NIP-42](https://github.com/nostr-protocol/nips/blob/master/42.md) and [NIP-45](https://github.com/nostr-protocol/nips/blob/master/45.md) introduce two new messaging types `AUTH` and `COUNT`.

The `AUTH` type is designed to facilitate a method by which clients can authenticate with a given relay. Whereas the `COUNT` type offers a method for relays to provide simple counts of events to clients (upon request). These are constructed in much the same way as the earlier message examples, by using the `RelayMessage` class in conjunction with the relevant methods `auth()` and `count()`. 

```javascript,ignore
{{#include ../../snippets/nostr/js/src/messages/relay.js:auth-message}}
```

```javascript,ignore
{{#include ../../snippets/nostr/js/src/messages/relay.js:count-message}}
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

## Error Messages

<custom-tabs category="lang">

<div slot="title">Rust</div>
<section>

TODO

</section>

<div slot="title">Python</div>
<section>

Finally, the `RelayMessageEnum` class also opens up two additional message types `NEG_ERR()` and `NEG_CODE()`. These do not form part of the standard protocol specification but do have specific uses when it comes to providing methods by which error messaging (or error codes) can be handled by relays. To construct these we need to first create them as instance of the `RelayMessageEnum` class and then pass these into a `RelayMessage` object using the `from_enum()` method.

```python,ignore
{{#include ../../snippets/nostr/python/src/messages/relay.py:neg-code}}
```

```python,ignore
{{#include ../../snippets/nostr/python/src/messages/relay.py:neg-msg}}
```

</section>

<div slot="title">JavaScript</div>
<section>

Not available currently with JavaScript bindings.

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
