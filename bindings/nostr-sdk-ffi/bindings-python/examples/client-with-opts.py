from nostr_sdk import Keys, Client, Options, EventBuilder
from datetime import timedelta

keys = Keys.generate()
print(keys.public_key().to_bech32())

# Change default send timeout
opts = Options().send_timeout(timedelta(seconds=10))
client = Client.with_opts(None, opts)

client.add_relay("wss://relay.damus.io")
client.connect()

event = EventBuilder.text_note("Hello from Rust Nostr Python bindings!", []).to_event(keys)
event_id = client.send_event(event)
print("Event sent:")
print(f" hex:    {event_id.to_hex()}")
print(f" bech32: {event_id.to_bech32()}")
