# Nostr SDK

## Description

Nostr protocol implementation, Relay, RelayPool, high-level client library, NWC client and more.

## Getting started

```shell
pip install nostr-sdk
```

```python
import asyncio
from datetime import timedelta
from nostr_sdk import *


async def main():
    # Init logger
    init_logger(LogLevel.INFO)

    # Initialize client without signer
    # client = Client()

    # Initialize with Keys signer
    keys = Keys.generate()
    signer = NostrSigner.keys(keys)
    client = Client(signer)

    # Initialize with NIP46 signer
    # app_keys = Keys.parse("..")
    # uri = NostrConnectUri.parse("bunker://.. or nostrconnect://..")
    # connect = NostrConnect(uri, app_keys, timedelta(seconds=60), None)
    # signer = NostrSigner.nostr_connect(connect)
    # client = Client(signer)

    # Add relays and connect
    await client.add_relay("wss://relay.damus.io")
    await client.add_relay("wss://nos.lol")
    await client.connect()

    # Send an event using the Nostr Signer
    builder = EventBuilder.text_note("Test from Rust Nostr Python!")
    await client.send_event_builder(builder)
    await client.set_metadata(Metadata().set_name("Testing Rust Nostr"))

    # Mine a POW event and sign it with custom keys
    custom_keys = Keys.generate()
    print("Mining a POW text note...")
    event = EventBuilder.text_note("Hello from Rust Nostr Python bindings!", []).pow(20).sign_with_keys(custom_keys)
    event_id = await client.send_event(event)
    print("Event sent:")
    print(f" hex:    {event_id.to_hex()}")
    print(f" bech32: {event_id.to_bech32()}")

    await asyncio.sleep(2.0)

    # Get events from relays
    print("Getting events from relays...")
    f = Filter().authors([keys.public_key(), custom_keys.public_key()])
    events = await client.fetch_events([f], timedelta(seconds=10))
    for event in events.to_vec():
        print(event.as_json())


asyncio.run(main())
```

More examples can be found [here](https://github.com/rust-nostr/nostr/tree/master/bindings/nostr-sdk-ffi/python/examples).

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
