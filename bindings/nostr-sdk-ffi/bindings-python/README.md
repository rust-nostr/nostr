# Nostr SDK - Python Package

The Python language bindings for [nostr-sdk](https://github.com/rust-nostr/nostr).

## Getting started

```shell
pip install nostr-sdk
```

```python
from nostr_sdk import Keys, Client, EventBuilder, Filter
import time

keys = Keys.generate()
print(keys.public_key().to_bech32())

client = Client(keys)

client.add_relay("wss://relay.damus.io")
client.connect()

print("Mining a POW text note...")
event = EventBuilder.new_text_note("Hello from Rust Nostr Python bindings!", []).to_pow_event(keys, 20)
event_id = client.send_event(event)
print("Event sent:")
print(f" hex:    {event_id.to_hex()}")
print(f" bech32: {event_id.to_bech32()}")

time.sleep(2.0)

print("Getting events from relays...")
filter = Filter().authors([keys.public_key().to_hex()])
events = client.get_events_of([filter], None)
for event in events:
    print(event.as_json())
```

More examples can be found in the [examples/](https://github.com/rust-nostr/nostr/tree/master/bindings/nostr-sdk-ffi/bindings-python/examples) directory.

## Supported NIPs

Look at <https://github.com/rust-nostr/nostr/tree/master/crates/nostr#supported-nips>

## State

**This library is in an ALPHA state**, things that are implemented generally work but the API will change in breaking ways.

## License

This project is distributed under the MIT software license - see the [LICENSE](https://github.com/rust-nostr/nostr/tree/master/LICENSE) file for details

## Donations

⚡ Tips: <https://getalby.com/p/yuki>

⚡ Lightning Address: yuki@getalby.com