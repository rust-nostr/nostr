import asyncio
from nostr_sdk import Keys, Client, ClientBuilder, Options, EventBuilder, Connection, ConnectionTarget, init_logger, LogLevel
from datetime import timedelta


async def main():
    init_logger(LogLevel.INFO)

    keys = Keys.generate()
    print(keys.public_key().to_bech32())

    # Configure client to use embedded tor for `.onion` relays
    connection = Connection().embedded_tor().target(ConnectionTarget.ONION)
    opts = Options().connection(connection).connection_timeout(timedelta(seconds=60))
    client = ClientBuilder().signer(keys).opts(opts).build()

    await client.add_relay("wss://relay.damus.io")
    await client.add_relay("ws://oxtrdevav64z64yb7x6rjg4ntzqjhedm5b5zjqulugknhzr46ny2qbad.onion")
    await client.add_relay("ws://2jsnlhfnelig5acq6iacydmzdbdmg7xwunm4xl6qwbvzacw4lwrjmlyd.onion")
    await client.connect()

    event = EventBuilder.text_note("Hello from rust-nostr Python bindings!")
    res = await client.send_event_builder(event)
    print("Event sent:")
    print(f" hex:    {res.id.to_hex()}")
    print(f" bech32: {res.id.to_bech32()}")
    print(f" Successfully sent to:    {res.output.success}")
    print(f" Failed to send to: {res.output.failed}")


if __name__ == '__main__':
    asyncio.run(main())
