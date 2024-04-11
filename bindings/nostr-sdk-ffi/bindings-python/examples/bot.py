from nostr_sdk import Client, NostrSigner, Keys, Event, UnsignedEvent, Filter, \
    HandleNotification, Timestamp, nip04_decrypt, UnwrappedGift, init_logger, LogLevel, Kind, KindEnum
import time

init_logger(LogLevel.DEBUG)

# sk = SecretKey.from_bech32("nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85")
# keys = Keys(sk)
# OR
keys = Keys.parse("nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85")

sk = keys.secret_key()
pk = keys.public_key()
print(f"Bot public key: {pk.to_bech32()}")

signer = NostrSigner.keys(keys)
client = Client(signer)

client.add_relay("wss://relay.damus.io")
client.add_relay("wss://nostr.mom")
client.add_relay("wss://nostr.oxtr.dev")
client.connect()

now = Timestamp.now()

nip04_filter = Filter().pubkey(pk).kind(Kind.from_enum(KindEnum.ENCRYPTED_DIRECT_MESSAGE())).since(now)
nip59_filter = Filter().pubkey(pk).kind(Kind.from_enum(KindEnum.GIFT_WRAP())).since(
    Timestamp.from_secs(now.as_secs() - 60 * 60 * 24 * 7))  # NIP59 have a tweaked timestamp (in the past)
client.subscribe([nip04_filter, nip59_filter], None)


class NotificationHandler(HandleNotification):
    def handle(self, relay_url, subscription_id, event: Event):
        print(f"Received new event from {relay_url}: {event.as_json()}")
        if event.kind().match_enum(KindEnum.ENCRYPTED_DIRECT_MESSAGE()):
            print("Decrypting NIP04 event")
            try:
                msg = nip04_decrypt(sk, event.author(), event.content())
                print(f"Received new msg: {msg}")
                client.send_direct_msg(event.author(), f"Echo: {msg}", event.id())
            except Exception as e:
                print(f"Error during content NIP04 decryption: {e}")
        elif event.kind().match_enum(KindEnum.GIFT_WRAP()):
            print("Decrypting NIP59 event")
            try:
                # Extract rumor
                unwrapped_gift = UnwrappedGift.from_gift_wrap(keys, event)
                sender = unwrapped_gift.sender()
                rumor: UnsignedEvent = unwrapped_gift.rumor()

                # Check timestamp of rumor
                if rumor.created_at().as_secs() >= now.as_secs():
                    if rumor.kind().match_enum(KindEnum.SEALED_DIRECT()):
                        msg = rumor.content()
                        print(f"Received new msg [sealed]: {msg}")
                        client.send_sealed_msg(sender, f"Echo: {msg}", None)
                    else:
                        print(f"{rumor.as_json()}")
            except Exception as e:
                print(f"Error during content NIP59 decryption: {e}")

    def handle_msg(self, relay_url, msg):
        None


abortable = client.handle_notifications(NotificationHandler())
# Optionally, to abort handle notifications look, call abortable.abort()

while True:
    time.sleep(5.0)
    # abortable.abort()
