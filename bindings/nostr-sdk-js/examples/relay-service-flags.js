const { Keys, Client, NostrSigner, loadWasmAsync, RelayServiceFlags, initLogger, LogLevel } = require("../");

async function main() {
    await loadWasmAsync();

    initLogger(LogLevel.info());

    try {
        // Generate random keys
        let keys = Keys.generate();
        let signer = NostrSigner.keys(keys);
        let client = new Client(signer);

        await client.addRelay("wss://relay.damus.io");
        await client.connect();

        // This note will be published
        await client.publishTextNote("My first text note from rust-nostr JS!", []);

        // Change relay service flags (remove write permission)
        let relay = await client.relay("wss://relay.damus.io");
        let flags = relay.flags();
        flags.remove(RelayServiceFlags.write()); // Use flags.add(..); to add a flag

        // This note will NOT be published
        await client.publishTextNote("Trying to send a second note", []);
    } catch (error) {
        console.log(error);
    }
}

main();