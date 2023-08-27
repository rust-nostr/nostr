from nostr_sdk import Keys, Client, Options, EventBuilder

keys = Keys.generate()
print(keys.public_key().to_bech32())

# Disable wait_for_ok option: the client will send events without waiting for `OK` confirmation from relays
opts = Options().wait_for_ok(False)
client = Client.with_opts(keys, opts)

client.add_relay("wss://relay.damus.io")
client.connect()

event = EventBuilder.new_text_note("Hello from Rust Nostr Python bindings!", []).to_event(keys)
event_id = client.send_event(event)
print("Event sent:")
print(f" hex:    {event_id.to_hex()}")
print(f" bech32: {event_id.to_bech32()}")
