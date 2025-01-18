import asyncio
from nostr_sdk import Client, Keys, Event, UnsignedEvent, Filter, \
    HandleNotification, Timestamp, UnwrappedGift, init_logger, LogLevel, Kind, KindStandard, NostrSigner


async def main():
    init_logger(LogLevel.DEBUG)

    # sk = SecretKey.from_bech32("nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85")
    # keys = Keys(sk)
    # OR
    keys = Keys.parse("nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85")

    pk = keys.public_key()
    print(f"Bot public key: {pk.to_bech32()}")

    signer = NostrSigner.keys(keys)
    client = Client(signer)

    await client.add_relay("wss://relay.damus.io")
    await client.add_relay("wss://nostr.mom")
    await client.add_relay("wss://nostr.oxtr.dev")
    await client.connect()

    now = Timestamp.now()

    nip59_filter = Filter().pubkey(pk).kind(Kind.from_std(KindStandard.GIFT_WRAP)).limit(0)
    await client.subscribe([nip59_filter], None)

    class NotificationHandler(HandleNotification):
        async def handle(self, relay_url, subscription_id, event: Event):
            print(f"Received new event from {relay_url}: {event.as_json()}")
            if event.kind().as_std() == KindStandard.GIFT_WRAP:
                print("Decrypting NIP59 event")
                try:
                    # Extract rumor
                    unwrapped_gift = await UnwrappedGift.from_gift_wrap(signer, event)
                    sender = unwrapped_gift.sender()
                    rumor: UnsignedEvent = unwrapped_gift.rumor()

                    # Check timestamp of rumor
                    if rumor.created_at().as_secs() >= now.as_secs():
                        if rumor.kind().as_std() == KindStandard.PRIVATE_DIRECT_MESSAGE:
                            msg = rumor.content()
                            print(f"Received new msg [sealed]: {msg}")
                            await client.send_private_msg(sender, f"Echo: {msg}", None)
                        else:
                            print(f"{rumor.as_json()}")
                except Exception as e:
                    print(f"Error during content NIP59 decryption: {e}")

        async def handle_msg(self, relay_url, msg):
            _var = None

    await client.handle_notifications(NotificationHandler())

    # To handle notifications and continue with code execution, use:
    # asyncio.create_task(client.handle_notifications(NotificationHandler()))

    # Keep up the script (if using the create_task)
    # while True:
    #   await asyncio.sleep(5)

if __name__ == '__main__':
    asyncio.run(main())
