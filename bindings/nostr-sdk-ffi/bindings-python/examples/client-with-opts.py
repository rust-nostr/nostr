from nostr_sdk import Keys, Client, Options, EventBuilder, Proxy, ProxyTarget, init_logger, LogLevel
from datetime import timedelta

init_logger(LogLevel.INFO)

keys = Keys.generate()
print(keys.public_key().to_bech32())

# Configure client to use proxy for `.onion` relays
proxy = Proxy("127.0.0.1:9050").target(ProxyTarget.ONION)
opts = (Options()
        .connection_timeout(timedelta(seconds=60))
        .send_timeout(timedelta(seconds=10))
        .proxy(proxy))
client = Client.with_opts(None, opts)

client.add_relays([
    "wss://relay.damus.io",
    "ws://oxtrdevav64z64yb7x6rjg4ntzqjhedm5b5zjqulugknhzr46ny2qbad.onion"
])
client.connect()

event = EventBuilder.text_note("Hello from Rust Nostr Python bindings!", []).to_event(keys)
event_id = client.send_event(event)
print("Event sent:")
print(f" hex:    {event_id.to_hex()}")
print(f" bech32: {event_id.to_bech32()}")
