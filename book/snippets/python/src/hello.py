# ANCHOR: full
import asyncio
from nostr_sdk import Keys, Client, EventBuilder, NostrSigner


async def hello():
    # ANCHOR: client
    keys = Keys.generate()
    signer = NostrSigner.keys(keys)
    client = Client(signer)
    # ANCHOR_END: client

    # ANCHOR: connect
    await client.add_relay("wss://relay.damus.io")
    await client.connect()
    # ANCHOR_END: connect

    # ANCHOR: publish
    builder = EventBuilder.text_note("Hello, rust-nostr!")
    output = await client.send_event_builder(builder)
    # ANCHOR_END: publish

    # ANCHOR: output
    print(f"Event ID: {output.id.to_bech32()}")
    print(f"Sent to: {output.success}")
    print(f"Not send to: {output.failed}")
    # ANCHOR_END: output

if __name__ == '__main__':
   asyncio.run(hello())
# ANCHOR_END: full
