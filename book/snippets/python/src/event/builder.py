# ANCHOR: full
import asyncio
from nostr_sdk import Keys, EventBuilder, Kind, Tag, NostrSigner, Timestamp


async def sign_and_print(signer: NostrSigner, builder: EventBuilder):
    # ANCHOR: sign
    event = await builder.sign(signer)
    # ANCHOR_END: sign

    print(event.as_json())


async def event_builder():
    keys = Keys.generate()
    signer = NostrSigner.keys(keys)

    # ANCHOR: standard
    builder1 = EventBuilder.text_note("Hello")
    # ANCHOR_END: standard

    await sign_and_print(signer, builder1)

    # ANCHOR: std-custom
    tag = Tag.alt("POW text-note")
    custom_timestamp = Timestamp.from_secs(1737976769)
    builder2 = EventBuilder.text_note("Hello with POW").tags([tag]).pow(20).custom_created_at(custom_timestamp)
    # ANCHOR_END: std-custom

    await sign_and_print(signer, builder2)

    # ANCHOR: custom
    kind = Kind(33001)
    builder3 = EventBuilder(kind, "My custom event")
    # ANCHOR_END: custom

    await sign_and_print(signer, builder3)

if __name__ == '__main__':
   asyncio.run(event_builder())
# ANCHOR_END: full
