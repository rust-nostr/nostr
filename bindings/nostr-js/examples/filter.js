const { Filter, loadWasmAsync, Timestamp } = require("../");

async function main() {
    await loadWasmAsync();

    const filter = new Filter().kind(4).until(Timestamp.now()).limit(10);
    console.log('filter', filter.asJson());
}

main();