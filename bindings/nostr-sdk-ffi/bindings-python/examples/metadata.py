from nostr_sdk import Metadata, Client, NostrSigner, Keys, Filter, PublicKey, Kind
from datetime import timedelta

keys = Keys.generate()

signer = NostrSigner.keys(keys)
client = Client(signer)

client.add_relay("wss://relay.damus.io")
client.connect()

# Set metadata
metadata = Metadata()\
    .set_name("username")\
    .set_display_name("My Username")\
    .set_about("Description")\
    .set_picture("https://example.com/avatar.png")\
    .set_banner("https://example.com/banner.png")\
    .set_nip05("username@example.com")\
    .set_lud16("yuki@getalby.com")

print(f"Setting profile metadata for {keys.public_key().to_bech32()}...")
print(metadata.as_json())
client.set_metadata(metadata)

# Get metadata
pk = PublicKey.from_bech32("npub1drvpzev3syqt0kjrls50050uzf25gehpz9vgdw08hvex7e0vgfeq0eseet")
print(f"\nGetting profile metadata for {pk.to_bech32()}...")
filter = Filter().kind(Kind(0)).author(pk).limit(1)
events = client.get_events_of([filter], timedelta(seconds=10))
for event in events:
    metadata = Metadata.from_json(event.content())
    print(f"Name: {metadata.get_name()}")
    print(f"NIP05: {metadata.get_nip05()}")
    print(f"LUD16: {metadata.get_lud16()}")