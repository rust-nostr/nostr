import asyncio
from nostr_sdk import Keys, Client, Options, EventBuilder, Connection, ConnectionTarget, init_logger, LogLevel
from datetime import timedelta


async def main():
    init_logger(LogLevel.INFO)

    keys = Keys.generate()
    print(keys.public_key().to_bech32())

    # Configure client to use proxy for `.onion` relays
    proxy = Connection().proxy("127.0.0.1:9050").target(ConnectionTarget.ONION)
    opts = (Options()
            .connection_timeout(timedelta(seconds=60))
            .send_timeout(timedelta(seconds=10))
            .proxy(proxy))
    client = Client.with_opts(None, opts)

    await client.add_relay("wss://relay.damus.io")
    await client.add_relay("ws://oxtrdevav64z64yb7x6rjg4ntzqjhedm5b5zjqulugknhzr46ny2qbad.onion")
    await client.connect()

    event = EventBuilder.text_note("Hello from Rust Nostr Python bindings!", []).to_event(keys)
    event_id = await client.send_event(event)
    print("Event sent:")
    print(f" hex:    {event_id.to_hex()}")
    print(f" bech32: {event_id.to_bech32()}")


if __name__ == '__main__':
    asyncio.run(main())
