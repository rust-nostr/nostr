const { Keys, Client, NostrSigner, Filter, Timestamp, nip04_decrypt, initLogger, LogLevel, loadWasmAsync } = require("../");

async function main() {
    await loadWasmAsync();

    try {
        initLogger(LogLevel.info());
    } catch (error) {
        console.log(error);
    }
    

    let keys = Keys.fromSkStr("nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85");

    let signer = NostrSigner.keys(keys);
    let client = new Client(signer);

    await client.addRelay("wss://relay.damus.io");
    await client.addRelay("wss://nos.lol");
    await client.addRelay("wss://nostr.oxtr.dev");

    await client.connect();

    const filter = new Filter().pubkey(keys.publicKey).kind(4).since(Timestamp.now());
    console.log('filter', filter.asJson());

    await client.subscribe([filter]); 

    const handle = {
        // Handle event
        handleEvent: async (relayUrl, subscriptionId, event) => {
            console.log("Received new event from ", relayUrl);
            if (event.kind == 4) {
                try {
                    let content = nip04_decrypt(keys.secretKey, event.pubkey, event.content);
                    console.log("Message:", content);
                    await client.sendDirectMsg(event.pubkey, "Echo: " + content);

                    if (content == "stop") {
                        return true
                    }
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