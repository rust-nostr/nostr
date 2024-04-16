# Nostr - Python Package

## Description

Python bindings of [nostr](https://github.com/rust-nostr/nostr) library.

If you're writing a typical Nostr client or bot, you may be interested in [nostr-sdk](https://pypi.org/project/nostr-sdk/).

## Getting started

```shell
pip install nostr-protocol
```

```python
from nostr_protocol import Keys, EventBuilder

keys = Keys.generate()
print(keys.secret_key().to_bech32())
print(keys.public_key().to_bech32())

print("Mining a POW text note...")
event = EventBuilder.text_note("Hello from Rust Nostr Python bindings!", []).to_pow_event(keys, 20)
print(event.as_json())
```

More examples can be found in the [examples/](https://github.com/rust-nostr/nostr/tree/master/bindings/nostr-ffi/bindings-python/examples) directory.

## Supported NIPs

Look at <https://github.com/rust-nostr/nostr/tree/master/crates/nostr#supported-nips>

## Book

Learn more about `rust-nostr` at <https://rust-nostr.org>.

## State

**This library is in an ALPHA state**, things that are implemented generally work but the API will change in breaking ways.

## Donations

`rust-nostr` is free and open-source. This means we do not earn any revenue by selling it. Instead, we rely on your financial support. If you actively use any of the `rust-nostr` libs/software/services, then please [donate](https://rust-nostr.org/donate).

## License

This project is distributed under the MIT software license - see the [LICENSE](https://github.com/rust-nostr/nostr/tree/master/LICENSE) file for details