from nostr_sdk import Keys, Client, EventBuilder, Filter
import time

keys = Keys.generate()
print(keys.public_key_bech32())

client = Client(keys)

client.add_relay("wss://relay.damus.io")
client.connect()

print("Mining a POW text note...")
event = EventBuilder.new_text_note("Hello from Rust Nostr Python bindings!", []).to_pow_event(keys, 20)
event_id = client.send_event(event)
print(f"Event sent: {event_id}")

time.sleep(2.0)

print("Getting events from relays...")
filter = Filter().authors([keys.public_key()])
events = client.get_events_of([filter], None)
for event in events:
    print(event.as_json())

