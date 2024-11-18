const { PublicKey, verifyNip05, loadWasmAsync } = require("../");

async function main() {
    await loadWasmAsync();
    
    let public_key = PublicKey.fromBech32("npub1drvpzev3syqt0kjrls50050uzf25gehpz9vgdw08hvex7e0vgfeq0eseet");
    if (await verifyNip05(public_key, "pay@yukikishimoto.com")) {
        console.log("Valid NIP05")
    } else {
        console.log("Invalid NIP05: " + error)
    }
}

main();
