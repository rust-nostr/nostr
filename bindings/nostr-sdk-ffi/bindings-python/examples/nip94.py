from nostr_sdk import Keys, Client, NostrSigner, FileMetadata

keys = Keys.generate()
print(keys.public_key().to_bech32())

signer = NostrSigner.keys(keys)
client = Client(signer)

client.add_relay("wss://relay.damus.io")
client.add_relay("ws://jgqaglhautb4k6e6i2g34jakxiemqp6z4wynlirltuukgkft2xuglmqd.onion", "127.0.0.1:9050")
client.connect()

try:
    metadata = FileMetadata("https://github.com/coinstr/coinstr/archive/refs/tags/v0.3.0.zip", "application/zip", "3951c152d38317e9ef2c095ddb280613e22b14b166f5fa5950d18773ac0a1d00")
    event_id = client.file_metadata("Coinstr Alpha Release v0.3.0", metadata)
    print("Event sent:")
    print(f" hex:    {event_id.to_hex()}")
    print(f" bech32: {event_id.to_bech32()}")
except Exception as e:
    print(f"{e}")
