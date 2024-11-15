import asyncio
from nostr_sdk import *


async def main():
    keys = Keys.generate()

    # Build a text note
    builder = EventBuilder.text_note("Note from rust-nostr python bindings")
    event = await builder.sign(keys)
    print(event.as_json())

    # Build a custom event
    kind = Kind(1234)
    content = "My custom content"
    builder = EventBuilder(kind, content)

    # Sign with generic signer
    event = await builder.sign(keys)
    print(f"Event: {event.as_json()}")

    # Sign specifically with keys
    event = builder.sign_with_keys(keys)
    print(f"Event: {event.as_json()}")

    # POW
    event = await builder.pow(24).sign(keys)
    print(f"POW event: {event.as_json()}")

    # Build unsigned event
    event = builder.build(keys.public_key())
    print(f"Event: {event.as_json()}")


if __name__ == '__main__':
    asyncio.run(main())
