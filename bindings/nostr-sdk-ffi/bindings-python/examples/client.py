from nostr_sdk import Keys, Client, NostrSigner, EventBuilder, Filter, Metadata, Nip46Signer, init_logger, LogLevel, \
    NostrConnectUri
from datetime import timedelta
import time

# Init logger
init_logger(LogLevel.INFO)

# Initialize client without signer
# client = Client(None)

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
client.add_relays(["wss://relay.damus.io", "wss://nos.lol"])
client.connect()

# Send an event using the Nostr Signer
builder = EventBuilder.text_note("Test from Rust Nostr Python!", [])
client.send_event_builder(builder)
client.set_metadata(Metadata().set_name("Testing Rust Nostr"))

# Mine a POW event and sign it with custom keys
custom_keys = Keys.generate() 
print("Mining a POW text note...")
event = EventBuilder.text_note("Hello from Rust Nostr Python bindings!", []).to_pow_event(custom_keys, 20)
event_id = client.send_event(event)
print("Event sent:")
print(f" hex:    {event_id.to_hex()}")
print(f" bech32: {event_id.to_bech32()}")

time.sleep(2.0)

# Get events from relays
print("Getting events from relays...")
f = Filter().authors([keys.public_key(), custom_keys.public_key()])
events = client.get_events_of([f], timedelta(seconds=10))
for event in events:
    print(event.as_json())
