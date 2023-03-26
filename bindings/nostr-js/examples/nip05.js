const { PublicKey, verifyNip05 } = require("../");

async function main() {
    let public_key = PublicKey.fromBech32("npub1drvpzev3syqt0kjrls50050uzf25gehpz9vgdw08hvex7e0vgfeq0eseet");
    let is_valid = await verifyNip05(public_key, "yuki@getalby.com");
    if (is_valid) {
        console.log("Valid NIP05")
    } else {
        console.log("Invalid NIP05")
    }
}

main();