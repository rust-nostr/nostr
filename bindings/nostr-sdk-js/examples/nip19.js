const { loadWasmAsync, Nip19Event } = require("../");

async function main() {
    await loadWasmAsync();

    let nevent = "nevent1qqsdhet4232flykq3048jzc9msmaa3hnxuesxy3lnc33vd0wt9xwk6szyqewrqnkx4zsaweutf739s0cu7et29zrntqs5elw70vlm8zudr3y24sqsgy";
    let nip19Event = Nip19Event.fromBech32(nevent);
    console.log(nip19Event.eventId().toHex())
}

main();