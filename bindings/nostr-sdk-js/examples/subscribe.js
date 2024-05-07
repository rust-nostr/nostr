const { PublicKey, Client, Filter, initLogger, LogLevel, loadWasmAsync } = require("../");

async function main() {
    await loadWasmAsync();

    try {
        initLogger(LogLevel.info());
    } catch (error) {
        console.log(error);
    }

    let client = new Client();

    await client.addRelay("wss://relay.damus.io");
    await client.addRelay("wss://nos.lol");
    await client.addRelay("wss://nostr.oxtr.dev");

    await client.connect();

    let publicKey = PublicKey.parse("npub1drvpzev3syqt0kjrls50050uzf25gehpz9vgdw08hvex7e0vgfeq0eseet");
    const filter = new Filter().pubkey(publicKey).kind(1).limit(10);
    console.log('filter', filter.asJson());

    // Subscribe to specific relays
    await client.subscribeTo(["wss://relay.damus.io"], [filter]);

    // Alternative way to subscribe to specific relay:
    // let relay = await client.relay("wss://relay.damus.io");
    // await relay.subscribe([filter], new SubscribeOptions());

    const handle = {
        // Handle event
        handleEvent: async (relayUrl, subscriptionId, event) => {
            console.log("Received new event from", relayUrl, ":", event.asJson());

            // Handle event
            // ...
        },
        // Handle relay message
        handleMsg: async (relayUrl, message) => {
            //console.log("Received message from", relayUrl, message.asJson());
        }
    };

    client.handleNotifications(handle);
}

main();