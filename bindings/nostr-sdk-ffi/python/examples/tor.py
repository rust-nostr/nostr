import asyncio
from nostr_sdk import Keys, ClientBuilder, Options, EventBuilder, Connection, ConnectionTarget, init_logger, LogLevel, NostrSigner


async def main():
    init_logger(LogLevel.INFO)

    keys = Keys.generate()
    print(keys.public_key().to_bech32())

    # Configure client to use a tor proxy for `.onion` relays
    connection = Connection().addr("127.0.0.1:9050").target(ConnectionTarget.ONION)
    opts = Options().connection(connection)
    signer = NostrSigner.keys(keys)
    client = ClientBuilder().signer(signer).opts(opts).build()

    await client.add_relay("wss://relay.damus.io")
    await client.add_relay("ws://oxtrdevav64z64yb7x6rjg4ntzqjhedm5b5zjqulugknhzr46ny2qbad.onion")
    await client.add_relay("ws://2jsnlhfnelig5acq6iacydmzdbdmg7xwunm4xl6qwbvzacw4lwrjmlyd.onion")
    await client.connect()

    event = EventBuilder.text_note("Hello from rust-nostr Python bindings!")
    output = await client.send_event_builder(event)
    print("Event sent:")
    print(f" hex:    {output.id.to_hex()}")
    print(f" bech32: {output.id.to_bech32()}")
    print(f" Successfully sent to:    {output.success}")
    print(f" Failed to send to: {output.failed}")


if __name__ == '__main__':
    asyncio.run(main())
