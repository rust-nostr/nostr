import asyncio
from datetime import timedelta
from nostr_sdk import Keys, Client, NostrSigner, EventBuilder, Filter, Metadata, EventSource, init_logger, LogLevel


async def main():
    # Init logger
    init_logger(LogLevel.INFO)

    # Initialize client without signer
    # client = Client()

    # Or, initialize with Keys signer
    keys = Keys.generate()
    signer = NostrSigner.keys(keys)

    # Or, initialize with NIP46 signer
    # app_keys = Keys.parse("..")
    # uri = NostrConnectUri.parse("bunker://.. or nostrconnect://..")
    # nip46 = Nip46Signer(uri, app_keys, timedelta(seconds=60), None)
    # signer = NostrSigner.nip46(nip46)

    client = Client(signer)

    # Add relays and connect
    await client.add_relay("wss://relay.damus.io")
    await client.add_relay("wss://nos.lol")
    await client.connect()

    # Send an event using the Nostr Signer
    builder = EventBuilder.text_note("Test from rust-nostr Python bindings!", [])
    await client.send_event_builder(builder)
    await client.set_metadata(Metadata().set_name("Testing rust-nostr"))

    # Mine a POW event and sign it with custom keys
    custom_keys = Keys.generate()
    print("Mining a POW text note...")
    event = EventBuilder.text_note("Hello from rust-nostr Python bindings!", []).to_pow_event(custom_keys, 20)
    output = await client.send_event(event)
    print("Event sent:")
    print(f" hex:    {output.id.to_hex()}")
    print(f" bech32: {output.id.to_bech32()}")
    print(f" Successfully sent to:    {output.success}")
    print(f" Failed to send to: {output.failed}")

    await asyncio.sleep(2.0)

    # Get events from relays
    print("Getting events from relays...")
    f = Filter().authors([keys.public_key(), custom_keys.public_key()])
    source = EventSource.relays(timedelta(seconds=10))
    events = await client.get_events_of([f], source)
    for event in events:
        print(event.as_json())


if __name__ == '__main__':
    asyncio.run(main())
