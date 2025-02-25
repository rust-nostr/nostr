const { Client, Filter, AdmitStatus, PublicKey, loadWasmAsync, initLogger, LogLevel, Kind, Duration } = require("../");

async function main() {
    await loadWasmAsync();

    initLogger(LogLevel.info());

    let mutedPublicKey = PublicKey.parse("npub1l2vyh47mk2p0qlsku7hg0vn29faehy9hy34ygaclpn66ukqp3afqutajft")
    let otherPublicKey = PublicKey.parse("npub1xtscya34g58tk0z605fvr788k263gsu6cy9x0mhnm87echrgufzsevkk5s")

    const filtering = {
        admitEvent: async (event) => {
            if (event.author.toBech32() === "npub1l2vyh47mk2p0qlsku7hg0vn29faehy9hy34ygaclpn66ukqp3afqutajft") {
                return AdmitStatus.Rejected
            } else {
                return AdmitStatus.Success
            }
        }
    }

    let client = Client.builder().admitPolicy(filtering).build();
    await client.addRelay("wss://relay.damus.io");
    await client.addRelay("wss://nos.lol");
    await client.addRelay("wss://nostr.oxtr.dev");
    await client.addRelay("wss://nostr.wine");

    await client.connect();

    const filter = new Filter().authors([mutedPublicKey, otherPublicKey]).kind(new Kind(0));
    let events = await client.fetchEvents(filter, Duration.fromSecs(10));
    console.log("Received", events.len(), "events");

    events.forEach((event) => {
        console.log(event.asJson())
    })

    // let list = events.toVec();
    // for (const event of list) {
    //     try {
    //         await client.sendEvent(event)
    //     } catch (e) {
    //         console.log(e)
    //     }
    // }
}

main();
