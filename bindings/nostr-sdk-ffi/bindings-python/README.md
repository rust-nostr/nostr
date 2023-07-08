# Nostr SDK - Python Package

The Python language bindings for [nostr-sdk](../../../crates/nostr-sdk/).

See the [package on PyPI](https://pypi.org/project/nostr-sdk/).  

## Getting started

```shell
pip install nostr-sdk
```

```python
from nostr_sdk import Keys, Client, EventBuilder

keys = Keys.generate()
pk = keys.public_key()
print(pk)

client = Client(keys)

client.add_relay("wss://relay.damus.io")
client.connect()

event = EventBuilder.new_text_note("Hello from Rust Nostr Python bindings!", []).to_pow_event(keys, 20)
client.send_event(event)
```

More examples can be found in the [examples/](https://github.com/rust-nostr/nostr/tree/master/bindings/nostr-sdk-ffi/bindings-python/examples) directory.

## Supported NIPs

Look at <https://github.com/rust-nostr/nostr/tree/master/crates/nostr#supported-nips>

## State

**This library is in an ALPHA state**, things that are implemented generally work but the API will change in breaking ways.

## License

This project is distributed under the MIT software license - see the [LICENSE](./LICENSE) file for details

## Donations

⚡ Tips: <https://getalby.com/p/yuki>

⚡ Lightning Address: yuki@getalby.com