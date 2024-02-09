const { loadWasmAsync, Client, NostrSigner, Nip07Signer } = require("../");

// NOTE: this code work only on browser!

async function main() {
    await loadWasmAsync();

    try {
        let nip07_signer = new Nip07Signer();
        let signer = NostrSigner.nip07(nip07_signer);
        let client = new Client(signer);

        await client.addRelay("wss://relay.damus.io");
        await client.addRelay("wss://nos.lol");
        await client.addRelay("wss://nostr.oxtr.dev");

        await client.connect();

        await client.publishTextNote("Test from Rust Nostr SDK JavaScript bindings with NIP07 signer!", []);
    } catch (error) {
        console.log(error) 
    }
}

main();