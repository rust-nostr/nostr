from nostr_sdk import Keys, Filter, ClientBuilder, NostrDatabase, NegentropyOptions, init_logger, LogLevel

init_logger(LogLevel.INFO)

keys = Keys.parse("nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85")
print(keys.public_key().to_bech32())

# Create/open SQLite database
database = NostrDatabase.sqlite("nostr.db")

# Create/open nostrdb database
# database = NostrDatabase.ndb("ndb")

client = ClientBuilder().database(database).build()

client.add_relay("wss://relay.damus.io")
client.add_relay("wss://atl.purplerelay.com")
client.connect()

# Negentropy reconciliation
f = Filter().author(keys.public_key())
opts = NegentropyOptions()
client.reconcile(f, opts)

# Query events from database
f = Filter().author(keys.public_key()).limit(10)
events = client.database().query([f])
for event in events:
    print(event.as_json())
