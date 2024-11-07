import asyncio
from nostr_sdk import PublicKey, Client, Filter, Kind, init_logger, LogLevel, Options, RelayFilteringMode
from datetime import timedelta


async def main():
    # Init logger
    init_logger(LogLevel.INFO)

    # Init client
    opts = Options().filtering_mode(RelayFilteringMode.WHITELIST)
    client = ClientBuilder().opts(opts).build()
    await client.add_relay("wss://relay.damus.io")
    await client.add_relay("wss://nos.lol")
    await client.connect()

    whitelisted_public_key = PublicKey.parse("npub1l2vyh47mk2p0qlsku7hg0vn29faehy9hy34ygaclpn66ukqp3afqutajft")
    not_whitelisted_public_key = PublicKey.parse("npub1xtscya34g58tk0z605fvr788k263gsu6cy9x0mhnm87echrgufzsevkk5s")

    # Mute public key
    filtering = client.filtering()
    await filtering.add_public_keys([whitelisted_public_key])

    # Get events
    f = Filter().authors([whitelisted_public_key, not_whitelisted_public_key]).kind(Kind(0))
    events = await client.fetch_events([f], timedelta(seconds=10))
    print(f"Received {events.__len__()} events")


if __name__ == '__main__':
    asyncio.run(main())
