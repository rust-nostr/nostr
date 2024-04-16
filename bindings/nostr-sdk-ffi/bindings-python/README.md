# Nostr SDK - Python Package

## Description

A high-level, [Nostr](https://github.com/nostr-protocol/nostr) client library.

If you're writing a typical Nostr client or bot, this is likely the crate you need.

However, the crate is designed in a modular way and depends on several other lower-level libraries. If you're attempting something more custom, you might be interested in these:

- [`nostr-protocol`](https://pypi.org/project/nostr-protocol/): Implementation of Nostr protocol

## Getting started

```shell
pip install nostr-sdk
```

```python
from nostr_sdk import Keys, Client, NostrSigner, EventBuilder, Filter, Metadata, Nip46Signer, init_logger, LogLevel, \
    NostrConnectUri
from datetime import timedelta
import time

# Init logger
init_logger(LogLevel.INFO)

# Initialize client without signer
# client = Client(None)

# Or, initialize with Keys signer
keys = Keys.generate()
signer = NostrSigner.keys(keys)

# Or, initialize with NIP46 signer
# app_keys = Keys.parse("..")
# uri = NostrConnectUri.parse("bunker://.. or nostrconnect://..")
# nip46 = Nip46Signer(uri, app_keys, timedelta(seconds=60), None)
# signer = NostrSigner.nip46(nip46)

client = Client(signer)

# Add relays and connect
client.add_relays(["wss://relay.damus.io", "wss://nos.lol"])
client.connect()

# Send an event using the Nostr Signer
builder = EventBuilder.text_note("Test from Rust Nostr Python!", [])
client.send_event_builder(builder)
client.set_metadata(Metadata().set_name("Testing Rust Nostr"))

# Mine a POW event and sign it with custom keys
custom_keys = Keys.generate()
print("Mining a POW text note...")
event = EventBuilder.text_note("Hello from Rust Nostr Python bindings!", []).to_pow_event(custom_keys, 20)
event_id = client.send_event(event)
print("Event sent:")
print(f" hex:    {event_id.to_hex()}")
print(f" bech32: {event_id.to_bech32()}")

time.sleep(2.0)

# Get events from relays
print("Getting events from relays...")
f = Filter().authors([keys.public_key(), custom_keys.public_key()])
events = client.get_events_of([f], timedelta(seconds=10))
for event in events:
    print(event.as_json())
```

More examples can be found at:

* https://github.com/rust-nostr/nostr/tree/master/bindings/nostr-ffi/bindings-python/examples
* https://github.com/rust-nostr/nostr/tree/master/bindings/nostr-sdk-ffi/bindings-python/examples

## Supported NIPs

Look at <https://github.com/rust-nostr/nostr/tree/master/crates/nostr#supported-nips>

## Book

Learn more about `rust-nostr` at <https://rust-nostr.org>.

## State

**This library is in an ALPHA state**, things that are implemented generally work but the API will change in breaking ways.

## Donations

`rust-nostr` is free and open-source. This means we do not earn any revenue by selling it. Instead, we rely on your financial support. If you actively use any of the `rust-nostr` libs/software/services, then please [donate](https://rust-nostr.org/donate).

## License

This project is distributed under the MIT software license - see the [LICENSE](https://github.com/rust-nostr/nostr/blob/master/LICENSE) file for details