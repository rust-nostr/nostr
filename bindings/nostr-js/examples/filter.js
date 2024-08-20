const { Filter, loadWasmAsync, Timestamp, Kind, Alphabet, SingleLetterTag } = require("../");

async function main() {
    await loadWasmAsync();

    const filter = new Filter().kind(new Kind(4)).until(Timestamp.now()).limit(10);
    console.log('filter', filter.asJson());

    // Custom tag
    let letter = SingleLetterTag.lowercase(Alphabet.J);
    const fisterCustom = new Filter().customTag(letter, ["custom-value"]);
    console.log('Custom filter', fisterCustom.asJson());
}

main();