const { loadWasmAsync, initLogger, LogLevel, SyncOptions, SyncDirection, Filter, Client, NostrDatabase, Kind } = require("../");

async function main() {
    await loadWasmAsync();

    initLogger(LogLevel.info());

    let db = await NostrDatabase.inMemory();
    let client = Client.builder().database(db).build();

    await client.addRelay("wss://relay.damus.io");

    await client.connect();

    let filter = new Filter().kind(new Kind(1)).limit(1000);
    let opts = new SyncOptions().direction(SyncDirection.Down);
    await client.sync(filter, opts);

    let f = new Filter().limit(2);
    let events = await db.query([f]);
    events.forEach((e) => {
        console.log(e.asJson())
    })
}

main();
