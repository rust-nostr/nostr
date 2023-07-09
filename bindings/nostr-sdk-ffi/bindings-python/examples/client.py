from nostr_sdk import Keys, Client, EventBuilder

keys = Keys.generate()
print(keys.public_key_bech32())

client = Client(keys)

client.add_relay("wss://relay.damus.io")
client.connect()

event = EventBuilder.new_text_note("Hello from Rust Nostr Python bindings!", []).to_pow_event(keys, 20)
event_id = client.send_event(event)
print(f"Event sent: {event_id}")

