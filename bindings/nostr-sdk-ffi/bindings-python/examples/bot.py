from nostr_sdk import Keys, Client, Event, EventBuilder, Filter, HandleNotification, timestamp, nip04_decrypt
import time

keys = Keys.from_bech32("nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85")
sk = keys.secret_key()
pk = keys.public_key()
print(f"Bot public key: {pk}")

client = Client(keys)

client.add_relay("wss://relay.damus.io")
client.add_relay("wss://offchain.pub")
client.add_relay("wss://nostr.oxtr.dev")
client.connect()

filter = Filter().pubkey(pk).kind(4).since(timestamp())
client.subscribe([filter])

class NotificationHandler(HandleNotification):
    def handle(self, relay_url, event):
        print(f"Received new event from {relay_url}: {event.as_json()}")
        if event.kind() == 4:
            print("Decrypting event")
            try:
                msg = nip04_decrypt(sk, event.pubkey(), event.content())
                print(f"Received new msg: {msg}")
                event = EventBuilder.new_encrypted_direct_msg(keys, event.pubkey(), f"Echo: {msg}").to_event(keys)
                client.send_event(event)
            except Exception as e:
                print(f"Error during content decryption: {e}")
    
client.handle_notifications(NotificationHandler())

while True:
    time.sleep(5.0)