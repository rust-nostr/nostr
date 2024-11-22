const { Keys, Client, NostrSigner, Filter, UnwrappedGift, initLogger, LogLevel, loadWasmAsync, EventBuilder } = require("../");

async function main() {
    await loadWasmAsync();

    try {
        initLogger(LogLevel.info());
    } catch (error) {
        console.log(error);
    }


    let keys = Keys.parse("nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85");

    let signer = NostrSigner.keys(keys);
    let client = new Client(signer);

    await client.addRelay("wss://relay.damus.io");
    await client.addRelay("wss://nos.lol");
    await client.addRelay("wss://nostr.oxtr.dev");

    await client.connect();

    const filter = new Filter().pubkey(keys.publicKey).kind(1059).limit(0); // Limit set to 0 to get only new events! Timestamp.now() CAN'T be used for gift wrap since the timestamps are tweaked!
    console.log('filter', filter.asJson());

    await client.subscribe([filter]);

    const handle = {
        // Handle event
        handleEvent: async (relayUrl, subscriptionId, event) => {
            console.log("Received new event from ", relayUrl);
            if (event.kind === 1059) {
                try {
                    let content = await UnwrappedGift.fromGiftWrap(signer, event);
                    let sender = content.sender;
                    let rumor = content.rumor;

                    if (rumor.content === "stop") {
                        return true
                    }
                    
                    await client.sendPrivateMsg(sender, "Echo: " + rumor.content);
                } catch (error) {
                    console.log("Impossible to decrypt DM:", error);
                }
            }
        },
        // Handle relay message
        handleMsg: async (relayUrl, message) => {
            //console.log("Received message from", relayUrl, message.asJson());
        }
    };

    client.handleNotifications(handle);
}

main();
