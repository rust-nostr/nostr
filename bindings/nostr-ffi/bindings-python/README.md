# Nostr - Python Package

The Python language bindings for [nostr](https://github.com/rust-nostr/nostr).

## Getting started

```shell
pip install nostr
```

```python
from nostr import Keys, EventBuilder

keys = Keys.generate()
print(keys.secret_key().to_bech32())
print(keys.public_key().to_bech32())

print("Mining a POW text note...")
event = EventBuilder.new_text_note("Hello from Rust Nostr Python bindings!", []).to_pow_event(keys, 20)
print(event.as_json())
```

More examples can be found in the [examples/](https://github.com/rust-nostr/nostr/tree/master/bindings/nostr-ffi/bindings-python/examples) directory.

## Supported NIPs

Look at <https://github.com/rust-nostr/nostr/tree/master/crates/nostr#supported-nips>

## State

**This library is in an ALPHA state**, things that are implemented generally work but the API will change in breaking ways.

## License

This project is distributed under the MIT software license - see the [LICENSE](https://github.com/rust-nostr/nostr/tree/master/LICENSE) file for details

## Donations

⚡ Tips: <https://getalby.com/p/yuki>

⚡ Lightning Address: yuki@getalby.com