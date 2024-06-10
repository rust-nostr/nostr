import asyncio

from nostr_sdk import Client
import time


async def main():
    client = Client(None)

    await client.add_relay("wss://relay.damus.io")
    await client.add_relay("wss://nostr.wine")
    await client.add_relay("wss://relay.nostr.info")
    await client.connect()

    while True:
        relays = await client.relays()
        for url, relay in relays.items():
            stats = relay.stats()
            print(f"Relay: {url}")
            print(f"Connected: {await relay.is_connected()}")
            print(f"Status: {await relay.status()}")
            print("Stats:")
            print(f"    Attempts: {stats.attempts()}")
            print(f"    Success: {stats.success()}")
            print(f"    Bytes sent: {stats.bytes_sent()}")
            print(f"    Bytes received: {stats.bytes_received()}")
            print(f"    Connected at: {stats.connected_at().to_human_datetime()}")
            if await stats.latency() is not None:
                print(f"    Latency: {stats.latency().total_seconds() * 1000} ms")

            print("###########################################")

        await asyncio.sleep(10.0)


if __name__ == '__main__':
    asyncio.run(main())
