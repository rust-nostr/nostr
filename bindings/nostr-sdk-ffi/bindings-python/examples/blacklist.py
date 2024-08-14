import asyncio
from nostr_sdk import PublicKey, Client, Filter, Kind, init_logger, LogLevel, EventSource
from datetime import timedelta


async def main():
    # Init logger
    init_logger(LogLevel.INFO)

    # Init client
    client = Client()
    await client.add_relays(["wss://relay.damus.io", "wss://nos.lol"])
    await client.connect()

    muted_public_key = PublicKey.parse("npub1l2vyh47mk2p0qlsku7hg0vn29faehy9hy34ygaclpn66ukqp3afqutajft")
    other_public_key = PublicKey.parse("npub1xtscya34g58tk0z605fvr788k263gsu6cy9x0mhnm87echrgufzsevkk5s")

    # Mute public key
    await client.mute_public_keys([muted_public_key])

    # Get events
    f = Filter().authors([muted_public_key, other_public_key]).kind(Kind(0))
    source = EventSource.relays(timedelta(seconds=10))
    events = await client.get_events_of([f], source)
    print(f"Received {events.__len__()} events")


if __name__ == '__main__':
    asyncio.run(main())
