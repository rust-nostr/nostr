import asyncio
from nostr_sdk import *
from nostr_sdk import uniffi_set_event_loop
from typing import List, Optional, Set


async def main():
    init_logger(LogLevel.INFO)

    uniffi_set_event_loop(asyncio.get_running_loop())

    # Example of custom in-memory database
    class MyDatabase(CustomNostrDatabase):
        def __init__(self):
            self.seen_event_ids = {}
            self.events = {}

        def backend(self) -> str:
            return "my-in-memory-backend"

        async def save_event(self, e: Event) -> bool:
            self.events[e.id()] = e
            return True

        async def check_id(self, event_id: "EventId") -> DatabaseEventStatus:
            if event_id in self.events:
                return DatabaseEventStatus.SAVED
            else:
                return DatabaseEventStatus.NOT_EXISTENT

        async def has_event_already_been_saved(self, event_id) -> bool:
            return event_id in self.events

        async def has_event_already_been_seen(self, event_id) -> bool:
            return event_id in self.seen_event_ids

        async def has_coordinate_been_deleted(self, coordinate, timestamp) -> bool:
            return False

        async def event_id_seen(self, event_id, relay_url: str):
            if event_id in self.seen_event_ids:
                self.seen_event_ids[event_id].add(relay_url)
            else:
                new_set = {relay_url}
                self.seen_event_ids[event_id] = new_set

        async def event_seen_on_relays(self, event_id) -> Optional[Set[str]]:
            return self.seen_event_ids.get(event_id)

        async def event_by_id(self, event_id) -> Event | None:
            return self.events.get(event_id, None)

        async def count(self, filters) -> int:
            return 0

        async def query(self, filters) -> List[Event]:
            # Fake algorithm
            return list(self.events.values())[:10]

        async def delete(self, filter):
            return

        async def wipe(self):
            self.seen_event_ids.clear()
            self.events.clear()

    my_db = MyDatabase()
    database = NostrDatabase.custom(my_db)
    client = ClientBuilder().database(database).build()

    await client.add_relay("wss://relay.damus.io")
    await client.add_relay("wss://atl.purplerelay.com")
    await client.connect()

    keys = Keys.parse("nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85")
    print(keys.public_key().to_bech32())

    # Negentropy reconciliation
    f = Filter().author(keys.public_key())
    opts = SyncOptions()
    await client.sync(f, opts)

    # Query events from database
    f = Filter().author(keys.public_key()).limit(10)
    events = await client.database().query([f])
    if len(events) == 0:
        print("Query not found any event")
    else:
        for event in events:
            print(event.as_json())


if __name__ == '__main__':
    asyncio.run(main())
