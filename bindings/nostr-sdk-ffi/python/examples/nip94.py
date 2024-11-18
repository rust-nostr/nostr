import asyncio
from nostr_sdk import Keys, Client, FileMetadata, EventBuilder


async def main():
    keys = Keys.generate()
    print(keys.public_key().to_bech32())

    client = Client(keys)

    await client.add_relay("wss://relay.damus.io")
    await client.connect()

    try:
        metadata = FileMetadata(
            "https://github.com/coinstr/coinstr/archive/refs/tags/v0.3.0.zip",
            "application/zip",
            "3951c152d38317e9ef2c095ddb280613e22b14b166f5fa5950d18773ac0a1d00"
        )
        builder = EventBuilder.file_metadata("Coinstr Alpha Release v0.3.0", metadata)
        output = await client.send_event_builder(builder)
        print("Event sent:")
        print(f" hex:    {output.id.to_hex()}")
        print(f" bech32: {output.id.to_bech32()}")
    except Exception as e:
        print(f"{e}")


if __name__ == '__main__':
    asyncio.run(main())
