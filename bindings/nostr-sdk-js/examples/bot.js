const { Keys, Client, ClientSigner, Filter, Timestamp, nip04_decrypt, loadWasmAsync } = require("../");

async function main() {
    await loadWasmAsync();

    let keys = Keys.fromSkStr("nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85");

    let signer = ClientSigner.keys(keys);
    let client = new Client(signer);

    await client.addRelay("wss://relay.damus.io");
    await client.addRelay("wss://nos.lol");
    await client.addRelay("wss://nostr.oxtr.dev");

    await client.connect();

    const filter = new Filter().pubkey(keys.publicKey).kind(BigInt(4)).since(Timestamp.now());
    console.log('filter', filter.asJson());

    await client.subscribe([filter]);

    const handleEvent = (relayUrl, event) => {
        // Handle event
        console.log("Received new event from", relayUrl);
        if (event.kind == BigInt(4)) {
            try {
                let content = nip04_decrypt(keys.secretKey, event.pubkey, event.content);
                console.log("Message:", content);
                client.sendDirectMsg(event.pubkey, "Echo: " + content)
            } catch (error) {
                console.log("Impossible to decrypt DM:", error);
            }
        }
    } 

    await client.handleEventNotifications(handleEvent);
}

main();