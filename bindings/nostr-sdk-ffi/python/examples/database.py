import asyncio
from nostr_sdk import Keys, Filter, ClientBuilder, NostrDatabase, SyncOptions, init_logger, LogLevel

init_logger(LogLevel.INFO)


async def main():
    keys = Keys.parse("nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85")
    print(keys.public_key().to_bech32())

    # Create/open LMDB database
    database = NostrDatabase.lmdb("nostr-lmdb")

    # NOT AVAILABLE ON WINDOWS AT THE MOMENT!
    # Create/open nostrdb database
    # database = NostrDatabase.ndb("ndb")

    client = ClientBuilder().database(database).build()

    await client.add_relay("wss://relay.damus.io")
    await client.add_relay("wss://atl.purplerelay.com")
    await client.connect()

    # Negentropy reconciliation
    f = Filter().author(keys.public_key())
    opts = SyncOptions()
    await client.sync(f, opts)

    # Query events from database
    f = Filter().author(keys.public_key()).limit(10)
    events = await client.database().query([f])
    for event in events:
        print(event.as_json())

if __name__ == '__main__':
    asyncio.run(main())
