const { loadWasmAsync, Client, EventBuilder, NostrSigner, BrowserSigner } = require("../");

// NOTE: this code work only on browser!

async function main() {
    await loadWasmAsync();

    try {
        let nip07_signer = new BrowserSigner();
        let signer = NostrSigner.nip07(nip07_signer);
        let client = new Client(signer);

        await client.addRelay("wss://relay.damus.io");
        await client.addRelay("wss://nos.lol");
        await client.addRelay("wss://nostr.oxtr.dev");

        await client.connect();
        
        let builder = EventBuilder.textNote("Test from rust-nostr JavaScript bindings with NIP07 signer!", []);
        await client.sendEventBuilder(builder);
    } catch (error) {
        console.log(error)
    }
}

async function main2() {
    try {
        let signer = new BrowserSigner();

        let public_key = await signer.getPublicKey();
        console.log(public_key.toBech32())

        let ciphertext = await signer.nip04Encrypt(public_key, "Test");
        console.log("NIP04: " + ciphertext);

        let unsigned = EventBuilder.textNote("Test", []).toUnsignedEvent(public_key);
        let event = await signer.signEvent(unsigned);
        console.log(event.asJson());
    } catch (error) {
        console.log(error)
    }
}

main();
main2();
