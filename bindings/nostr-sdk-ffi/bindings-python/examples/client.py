from nostr_sdk import Keys, Client, EventBuilder, Filter
from datetime import timedelta
import time

keys = Keys.generate()
print(keys.public_key().to_bech32())

client = Client(keys)

client.add_relay("wss://relay.damus.io", None)
client.connect()

print("Mining a POW text note...")
event = EventBuilder.new_text_note("Hello from Rust Nostr Python bindings!", []).to_pow_event(keys, 20)
event_id = client.send_event(event)
print("Event sent:")
print(f" hex:    {event_id.to_hex()}")
print(f" bech32: {event_id.to_bech32()}")

time.sleep(2.0)

print("Getting events from relays...")
filter = Filter().authors([keys.public_key()])
events = client.get_events_of([filter], timedelta(seconds=10))
for event in events:
    print(event.as_json())
