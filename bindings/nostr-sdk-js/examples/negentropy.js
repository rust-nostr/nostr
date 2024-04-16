const { loadWasmAsync, initLogger, LogLevel, NegentropyOptions, NegentropyDirection, Filter, Client, NostrDatabase } = require("../");

// NOTE: this code work only on browser (due to indexeddb)!

async function main() {
    await loadWasmAsync();

    initLogger(LogLevel.info());

    let db = await NostrDatabase.indexeddb("js-test");
    let client = Client.builder().database(db).build();

    await client.addRelay("wss://relay.damus.io");

    await client.connect();

    let direction = NegentropyDirection.Down;
    let opts = new NegentropyOptions().direction(direction);
    let filter = new Filter().kind(1).limit(1000);
    await client.reconcile(filter, opts);
}

main();