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
from nostr_sdk import Keys, Client, ClientSigner, EventBuilder, Filter, Metadata, Nip46Signer, init_logger, LogLevel
from datetime import timedelta
import time

# Init logger
init_logger(LogLevel.INFO)

# Initialize client without signer
# client = Client(None)

# Or, initialize with Keys signer
keys = Keys.generate()
signer = ClientSigner.keys(keys)
client = Client(signer)

# Or, initialize with NIP46 signer
# app_keys = Keys.generate()
# nip46 = Nip46Signer("wss://relay.damus.io", app_keys, None)
#signer = ClientSigner.nip46(nip46)
# client = Client(signer)

# Add a single relay
client.add_relay("wss://relay.damus.io")

# Add multiple relays
client.add_relays(["wss://relay.damus.io", "wss://nos.lol"])

# Connect
client.connect()

# Send an event using the Client Signer
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
filter = Filter().authors([keys.public_key(), custom_keys.public_key()])
events = client.get_events_of([filter], timedelta(seconds=10))
for event in events:
    print(event.as_json())
```

More examples can be found at:

* https://github.com/rust-nostr/nostr/tree/master/bindings/nostr-ffi/bindings-python/examples
* https://github.com/rust-nostr/nostr/tree/master/bindings/nostr-sdk-ffi/bindings-python/examples

## Supported NIPs

Look at <https://github.com/rust-nostr/nostr/tree/master/crates/nostr#supported-nips>

## State

**This library is in an ALPHA state**, things that are implemented generally work but the API will change in breaking ways.

## License

This project is distributed under the MIT software license - see the [LICENSE](https://github.com/rust-nostr/nostr/tree/master/LICENSE) file for details

## Donations

⚡ Tips: <https://getalby.com/p/yuki>

⚡ Lightning Address: yuki@getalby.com