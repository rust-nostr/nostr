## Client Message

One of the biggest strengths of the Nostr network is the almost limitless possibilities for interoperable user-facing applications. 
In protocol terminology these applications are often referred to as Clients. 
Where relays provide a data housing mechanism the clients get that data in front of users in myriad of wild and wonderful ways. 
Clients use WebSockets as a means to connect to relays and pass relevant data back and forth around the network. 
In accordance with the protocol base specification ([NIP-01](https://github.com/nostr-protocol/nips/blob/master/01.md#from-client-to-relay-sending-events-and-creating-subscriptions)) there are 3 main types of messages which clients construct and pass to relays as JSON arrays. 
This section is concerned with the construction of these message objects using the [Client Message Module](https://docs.rs/nostr/latest/nostr/message/client/index.html).

### JSON de/serialization

<custom-tabs category="lang">

<div slot="title">Rust</div>
<section>

TODO

</section>

<div slot="title">Python</div>
<section>

The `ClientMessage` class easily handles the construction of the 3 main message types `EVENT`, `REQ`, and `CLOSE`. 
In the examples below we can utilize the relevant class methods `event()`, `req()` and `close()`, respectively, to create the client message objects.

Once we have the `ClientMessage` objects we can use the `as_enum()` or `as_json()` methods to present their content. 
Note that when using `as_enum()` we unlock some additional methods associated with the `ClientMessageEnum` class. 
These allow for logical tests to be performed to establish the type of message object being assessed (for example, `is_req()` will return a bool result assessing if the object represents an `REQ` message type).  

```python,ignore
{{#include ../../../snippets/python/src/messages/client.py:event-message}}
```

Note that when constructing a `REQ` we want to pass through a `Filter` object which will allow the relay to return data meeting a given set of criteria. 
Please jump to the [Filter](filters.md) section for more details on how to construct these objects. 

```python,ignore
{{#include ../../../snippets/python/src/messages/client.py:req-message}}
```

```python,ignore
{{#include ../../../snippets/python/src/messages/client.py:close-message}}
```

When presented with a client message object as either a JSON or an instance of the `ClientMessageEnum` class we can parse these data using the `from_json()` or `from_enum()` methods, respectively.

```python,ignore
{{#include ../../../snippets/python/src/messages/client.py:parse-message}}
```

</section>

<div slot="title">JavaScript</div>
<section>

The `ClientMessage` class easily handles the construction of the 3 main message types `EVENT`, `REQ`, and `CLOSE`. 
In the examples below we can utilize the relevant class methods `event()`, `req()` and `close()`, respectively, to create the client message objects.

Once we have the `ClientMessage` objects we can use the `asJson()` method to present their content. 


```typescript,ignore
{{#include ../../../snippets/js/src/messages/client.ts:event-message}}
```

Note that when constructing a `REQ` we want to pass through a `Filter` object which will allow the relay to return data meeting a given set of criteria. 
Please jump to the [Filter](filters.md) section for more details on how to construct these objects. 

```typescript,ignore
{{#include ../../../snippets/js/src/messages/client.ts:req-message}}
```

```typescript,ignore
{{#include ../../../snippets/js/src/messages/client.ts:close-message}}
```

When presented with a client message object as either a JSON using the `fromJson()` method.

```typescript,ignore
{{#include ../../../snippets/js/src/messages/client.ts:parse-message}}
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

### Authorization and Count Messages

<custom-tabs category="lang">

<div slot="title">Rust</div>
<section>

TODO

</section>

<div slot="title">Python</div>
<section>

As an extension of the client messaging section of the protocol [NIP-42](https://github.com/nostr-protocol/nips/blob/master/42.md) and [NIP-45](https://github.com/nostr-protocol/nips/blob/master/45.md) introduce two new messaging types `AUTH` and `COUNT`.

The `AUTH` type is designed to facilitate a method by which clients can authenticate with a given relay. 
Whereas the `COUNT` type offers a method for clients can request simple counts of events from relays. 
These are constructed in much the same way as the earlier message examples, by using the `ClientMessage` class in conjunction with the relevant methods `auth()` and `count()`. 
As before the `as_enum()` method can be used to unlock logical test methods (e.g., `is_auth()`) associated with these message objects.

```python,ignore
{{#include ../../../snippets/python/src/messages/client.py:auth-message}}
```

Note that `COUNT` is effectively a specific type of `REQ` message therefore it utilizes the `Filter` object in constructing the criteria which should be used by the relay to return the count value.

```python,ignore
{{#include ../../../snippets/python/src/messages/client.py:count-message}}
```

</section>

<div slot="title">JavaScript</div>
<section>

As an extension of the client messaging section of the protocol [NIP-42](https://github.com/nostr-protocol/nips/blob/master/42.md) and [NIP-45](https://github.com/nostr-protocol/nips/blob/master/45.md) introduce two new messaging types `AUTH` and `COUNT`.

The `AUTH` type is designed to facilitate a method by which clients can authenticate with a given relay. 
Whereas the `COUNT` type offers a method for clients can request simple counts of events from relays. 
These are constructed in much the same way as the earlier message examples, by using the `ClientMessage` class in conjunction with the relevant methods `auth()` and `count()`. 

```typescript,ignore
{{#include ../../../snippets/js/src/messages/client.ts:auth-message}}
```

Note that `COUNT` is effectively a specific type of `REQ` message therefore it utilizes the `Filter` object in constructing the criteria which should be used by the relay to return the count value.

```typescript,ignore
{{#include ../../../snippets/js/src/messages/client.ts:count-message}}
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

### Negentropy Messages

<custom-tabs category="lang">

<div slot="title">Rust</div>
<section>

TODO

</section>

<div slot="title">Python</div>
<section>

Finally, the `ClientMessageEnum` class also opens up three additional message types `NEG_OPEN()`, `NEG_CLOSE()` and `NEG_MSG()`. 
These do not form part of the standard protocol specification but instead form part of an additional protocol [Negentropy](https://github.com/hoytech/negentropy) for handling set-reconciliation.

To construct these we need to first create them as instance of the `ClientMessageEnum` class and then pass these into a `ClientMessage` object using the `from_enum()` method.

```python,ignore
{{#include ../../../snippets/python/src/messages/client.py:neg-open}}
```

```python,ignore
{{#include ../../../snippets/python/src/messages/client.py:neg-close}}
```

```python,ignore
{{#include ../../../snippets/python/src/messages/client.py:neg-msg}}
```

</section>

<div slot="title">JavaScript</div>
<section>

Not currently available in the Javascript Bindings.

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
