from nostr_sdk import Keys, Client, EventBuilder

keys = Keys.generate()
pk = keys.public_key()
print(pk)

client = Client(keys)

client.add_relay("wss://relay.damus.io")
client.connect()

event = EventBuilder.new_text_note("Hello from Rust Nostr Python bindings!", []).to_pow_event(keys, 20)
client.send_event(event)
