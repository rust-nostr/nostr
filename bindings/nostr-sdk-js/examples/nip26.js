const { Keys, signDelegation, verifyDelegationSignature, PublicKey, loadWasmSync } = require("../");

function main() {
    loadWasmSync();
    
    // Generate new random keys
    let keys = Keys.generate();
    let delegatee = PublicKey.fromBech32("npub1gae33na4gfaeelrx48arwc2sc8wmccs3tt38emmjg9ltjktfzwtqtl4l6u");
    let conditions = "kind=1";
    let signature = signDelegation(keys, delegatee, conditions);
    console.log("Signature: ", signature);

    if (verifyDelegationSignature(keys.publicKey, delegatee, conditions, signature)) {
        console.log("Valid signature")
    } else {
        console.log("Invalid signature")
    }
}

main();