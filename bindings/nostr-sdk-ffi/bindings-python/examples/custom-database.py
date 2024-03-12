from nostr_sdk import Keys, Client, EventBuilder, Filter, ClientBuilder, CustomNostrDatabase, NostrDatabase, NegentropyOptions, Event, EventId, init_logger, LogLevel
from datetime import timedelta
import time
from typing import List, Optional, Set, Dict, Tuple

init_logger(LogLevel.INFO)

# Example of custom in-memory database
class MyDatabase(CustomNostrDatabase):
    def __init__(self):
        self.seen_event_ids = {}
        self.events = {}

    def backend(self) -> str:
        return "my-in-memory-backend"

    def save_event(self, event: Event) -> bool:
        self.events[event.id()] = event
        return True

    def has_event_already_been_saved(self, event_id) -> bool:
        return event_id in self.events

    def has_event_already_been_seen(self, event_id) -> bool:
        return event_id in self.seen_event_ids

    def has_event_id_been_deleted(self, event_id) -> bool:
        return False

    def has_coordinate_been_deleted(self, coordinate, timestamp) -> bool:
        return False
    
    def event_id_seen(self, event_id, relay_url: str):
        if event_id in self.seen_event_ids:
            self.seen_event_ids[event_id].add(relay_url)
        else:
            new_set = {relay_url}
            self.seen_event_ids[event_id] = new_set

    def event_seen_on_relays(self, event_id) -> Optional[Set[str]]:
        return self.seen_event_ids.get(event_id)

    def event_by_id(self, event_id) -> Event:
        return self.events.get(event_id, None)

    def count(self, filters) -> int:
        return 0

    def query(self, filters) -> List[Event]:
        # Fake algorithm
        return list(self.events.values())[:10]

    def delete(self, filter):
        return

    def wipe(self):
        self.seen_event_ids.clear()
        self.events.clear()

my_db = MyDatabase()
database = NostrDatabase.custom(my_db)
client = ClientBuilder().database(database).build()

client.add_relay("wss://relay.damus.io")
client.add_relay("wss://atl.purplerelay.com")
client.connect()

keys = Keys.parse("nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85")
print(keys.public_key().to_bech32())

# Negentropy reconciliation
filter = Filter().author(keys.public_key())
opts = NegentropyOptions()
client.reconcile(filter, opts)

# Query events from database
filter = Filter().author(keys.public_key()).limit(10)
events = client.database().query([filter])
if len(events) == 0:
    print("Query not found any event")
else:
    for event in events:
        print(event.as_json())
