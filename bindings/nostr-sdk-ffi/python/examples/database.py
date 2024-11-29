import asyncio
from nostr_sdk import *


async def main():
    init_logger(LogLevel.INFO)

    keys = Keys.parse("nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85")
    print(keys.public_key().to_bech32())

    # Create/open LMDB database
    database = NostrDatabase.lmdb("nostr-lmdb")

    client = ClientBuilder().database(database).build()

    await client.add_relay("wss://relay.damus.io")
    await client.connect()

    # Negentropy reconciliation
    f = Filter().author(keys.public_key())
    opts = SyncOptions()
    await client.sync(f, opts)

    # Query events from database
    f = Filter().author(keys.public_key()).limit(10)
    events = await client.database().query([f])
    for event in events.to_vec():
        print(event.as_json())

if __name__ == '__main__':
    asyncio.run(main())
