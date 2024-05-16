from nostr_sdk import PublicKey, Client, Filter, Kind, init_logger, LogLevel
from datetime import timedelta
import time

# Init logger
init_logger(LogLevel.INFO)

# Init client
client = Client()
client.add_relay("wss://relay.damus.io")
client.add_relay("wss://nos.lol")
client.connect()

muted_public_key = PublicKey.parse("npub1l2vyh47mk2p0qlsku7hg0vn29faehy9hy34ygaclpn66ukqp3afqutajft")
other_public_key = PublicKey.parse("npub1xtscya34g58tk0z605fvr788k263gsu6cy9x0mhnm87echrgufzsevkk5s")

# Mute public key
client.mute_public_keys([muted_public_key])

# Get events
f = Filter().authors([muted_public_key, other_public_key]).kind(Kind(0))
events = client.get_events_of([f], timedelta(seconds=10))
print(f"Received {events.__len__()} events")