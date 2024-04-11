from nostr_sdk import Client
import time

client = Client(None)

client.add_relay("wss://relay.damus.io")
client.add_relay("wss://nostr.wine")
client.add_relay("wss://relay.nostr.info")
client.connect()

while True:
    for url, relay in client.relays().items():
        stats = relay.stats()
        print(f"Relay: {url}")
        print(f"Connected: {relay.is_connected()}")
        print(f"Status: {relay.status()}")
        print("Stats:")
        print(f"    Attempts: {stats.attempts()}")
        print(f"    Success: {stats.success()}")
        print(f"    Bytes sent: {stats.bytes_sent()}")
        print(f"    Bytes received: {stats.bytes_received()}")
        print(f"    Connected at: {stats.connected_at().to_human_datetime()}")
        if stats.latency() is not None:
            print(f"    Latency: {stats.latency().total_seconds() * 1000} ms")

        print("###########################################")

    time.sleep(10.0)