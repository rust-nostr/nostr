const { loadWasmAsync, Nip07Signer } = require("../");

// NOTE: this code work only on browser!

async function main() {
    await loadWasmAsync();

    try {
        let signer = new Nip07Signer();

        let public_key = await signer.getPublicKey();
        console.log(public_key.toBech32())

        let ciphertext = await signer.nip04Encrypt(public_key, "Test"); 
        console.log("NIP04: " + ciphertext);

        let unsigned = EventBuilder.newTextNote("Test", []).toUnsignedEvent(public_key);
        let event = await signer.signEvent(unsigned);
        console.log(event.asJson());
    } catch (error) {
        console.log(error) 
    }
}

main();