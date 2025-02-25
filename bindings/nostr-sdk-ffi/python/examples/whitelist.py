import asyncio
from nostr_sdk import PublicKey, ClientBuilder, Filter, Kind, init_logger, LogLevel, AdmitPolicy, AdmitStatus, Event, \
    uniffi_set_event_loop
from datetime import timedelta

class WoT(AdmitPolicy):
    def __init__(self):
        self.allowed_public_keys = set()

    def allow(self, pk: PublicKey):
        self.allowed_public_keys.add(pk)

    async def admit_event(self, relay_url: str, subscription_id: str, event: Event) -> AdmitStatus:
        if event.author() in self.allowed_public_keys:
            return AdmitStatus.SUCCESS
        else:
            return AdmitStatus.REJECTED

async def main():
    uniffi_set_event_loop(asyncio.get_running_loop())

    # Init logger
    init_logger(LogLevel.INFO)

    whitelisted_public_key = PublicKey.parse("npub1l2vyh47mk2p0qlsku7hg0vn29faehy9hy34ygaclpn66ukqp3afqutajft")
    not_whitelisted_public_key = PublicKey.parse("npub1xtscya34g58tk0z605fvr788k263gsu6cy9x0mhnm87echrgufzsevkk5s")

    wot = WoT()
    wot.allow(whitelisted_public_key)

    # Init client
    client = ClientBuilder().admit_policy(wot).build()
    await client.add_relay("wss://relay.damus.io")
    await client.connect()

    # Get events
    f = Filter().authors([whitelisted_public_key, not_whitelisted_public_key]).kind(Kind(0))
    events = await client.fetch_events(f, timedelta(seconds=10))
    print(f"Received {events.len()} events")


if __name__ == '__main__':
    asyncio.run(main())
