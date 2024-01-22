const { Filter, loadWasmAsync, Timestamp, Alphabet } = require("../");

async function main() {
    await loadWasmAsync();

    const filter = new Filter().kind(4).until(Timestamp.now()).limit(10);
    console.log('filter', filter.asJson());

    // Custom tag
    const fisterCustom = new Filter().customTag(Alphabet.J, ["custom-value"]);
    console.log('Custom filter', fisterCustom.asJson());
}

main();