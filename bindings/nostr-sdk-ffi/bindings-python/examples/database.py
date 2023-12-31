from nostr_sdk import Keys, Client, EventBuilder, Filter, ClientBuilder, NostrDatabase
from datetime import timedelta
import time

keys = Keys.from_sk_str("nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85")
print(keys.public_key().to_bech32())

database = NostrDatabase.sqlite("nostr.db")
client = ClientBuilder().database(database).build()

client.add_relay("wss://relay.damus.io")
client.add_relay("wss://atl.purplerelay.com")
client.connect()

# Negentropy reconciliation
filter = Filter().author(keys.public_key())
client.reconcile(filter)

# Query events from database
filter = Filter().author(keys.public_key()).limit(10)
events = client.database().query([filter])
for event in events:
    print(event.as_json())
