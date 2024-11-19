# ANCHOR: full
from nostr_sdk import Keys, Client, EventBuilder


async def hello():
    # ANCHOR: client
    keys = Keys.generate()
    client = Client(keys)
    # ANCHOR_END: client

    # ANCHOR: connect
    await client.add_relay("wss://relay.damus.io")

    await client.connect()
    # ANCHOR_END: connect

    # ANCHOR: publish
    builder = EventBuilder.text_note("Hello, rust-nostr!", [])
    await client.send_event_builder(builder)
    # ANCHOR_END: publish

# ANCHOR_END: full
