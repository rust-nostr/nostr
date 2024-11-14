const { Keys, Client, NostrSigner, loadWasmAsync, RelayServiceFlags, initLogger, LogLevel } = require("../");
const {EventBuilder} = require("../pkg/nostr_sdk_js");

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
        let builder = EventBuilder.textNote("My first text note from rust-nostr JS!", []);
        await client.sendEventBuilder(builder);

        // Change relay service flags (remove write permission)
        let relay = await client.relay("wss://relay.damus.io");
        let flags = relay.flags();
        flags.remove(RelayServiceFlags.write()); // Use flags.add(..); to add a flag

        // This note will NOT be published
        let builder2 = EventBuilder.textNote("Trying to send a second note", []);
        await client.sendEventBuilder(builder2);
    } catch (error) {
        console.log(error);
    }
}

main();