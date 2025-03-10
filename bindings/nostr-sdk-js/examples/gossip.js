const { Keys, Client, NostrSigner, PublicKey, EventBuilder, loadWasmAsync, initLogger, Gossip, LogLevel, Tag } = require("../");

async function main() {
    await loadWasmAsync();

    initLogger(LogLevel.info());

    let keys = Keys.parse("nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85");
    let signer = NostrSigner.keys(keys);

    let gossip = Gossip.inMemory()

    let client = Client.builder().signer(signer).gossip(gossip).build();

    await client.addDiscoveryRelay("wss://relay.damus.io");
    await client.addDiscoveryRelay("wss://purplepag.es");

    await client.connect();

    let pk = PublicKey.parse("npub1drvpzev3syqt0kjrls50050uzf25gehpz9vgdw08hvex7e0vgfeq0eseet");

    let builder = EventBuilder.textNote(
        "Hello world nostr:npub1drvpzev3syqt0kjrls50050uzf25gehpz9vgdw08hvex7e0vgfeq0eseet",
    ).tags([Tag.publicKey(pk)]);
    let output = await client.sendEventBuilder(builder);
    console.log("Event ID", output.id.toBech32());
    console.log("Successfully sent to:", output.success);
    console.log("Failed to sent to:", output.failed);
}

main();