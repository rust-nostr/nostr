import asyncio
from nostr_sdk import Metadata, Client, Keys, Filter, PublicKey, Kind, NostrSigner, KindStandard
from datetime import timedelta


async def main():
    keys = Keys.generate()

    signer = NostrSigner.keys(keys)
    client = Client(signer)

    await client.add_relay("wss://relay.damus.io")
    await client.connect()

    # Set metadata
    metadata = Metadata() \
        .set_name("username") \
        .set_display_name("My Username") \
        .set_about("Description") \
        .set_picture("https://example.com/avatar.png") \
        .set_banner("https://example.com/banner.png") \
        .set_nip05("username@example.com") \
        .set_lud16("pay@yukikishimoto.com")

    print(f"Setting profile metadata for {keys.public_key().to_bech32()}...")
    print(metadata.as_json())
    await client.set_metadata(metadata)

    # Get metadata
    pk = PublicKey.parse("npub1drvpzev3syqt0kjrls50050uzf25gehpz9vgdw08hvex7e0vgfeq0eseet")
    print(f"\nGetting profile metadata for {pk.to_bech32()}...")
    f = Filter().kind(Kind.from_std(KindStandard.METADATA)).author(pk).limit(1)
    events = await client.fetch_events([f], timedelta(seconds=10))
    for event in events.to_vec():
        metadata = Metadata.from_json(event.content())
        print(f"Name: {metadata.get_name()}")
        print(f"NIP05: {metadata.get_nip05()}")
        print(f"LUD16: {metadata.get_lud16()}")


if __name__ == '__main__':
    asyncio.run(main())
