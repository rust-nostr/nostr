const { Keys, Client, NostrSigner, Metadata, EventId, PublicKey, EventBuilder, loadWasmAsync, initLogger, LogLevel, Kind } = require("../");

async function main() {
    await loadWasmAsync();

    initLogger(LogLevel.info());

    // Generate random keys
    let keys = Keys.parse("nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85");

    // Hex keys
    console.log("Public key (hex): ", keys.publicKey.toHex());
    console.log("Secret key (hex): ", keys.secretKey.toHex());

    // Bech32 keys
    console.log("Public key (bech32): ", keys.publicKey.toBech32());
    console.log("Secret key (bech32): ", keys.secretKey.toBech32());

    let signer = NostrSigner.keys(keys);
    let client = new Client(signer);
    await client.addRelay("wss://relay.damus.io");
    await client.addRelay("wss://nos.lol");
    await client.addRelay("wss://nostr.oxtr.dev");
    await client.addRelay("wss://nostr.wine");

    await client.connect();

    let metadata = new Metadata()
        .name("username")
        .displayName("My Username")
        .about("Description")
        .picture("https://example.com/avatar.png")
        .banner("https://example.com/banner.png")
        .nip05("username@example.com")
        .lud16("pay@yukikishimoto.com");
    
    await client.setMetadata(metadata);

    let b = EventBuilder.textNote("My first text note from rust-nostr WASM!", []);
    let output = await client.sendEventBuilder(b);
    console.log("Event ID", output.id.toBech32());
    console.log("Successfully sent to:", output.output.success);
    console.log("Failed to sent to:", output.output.failed);

    // Send custom event
    let event_id = EventId.fromBech32("note1z3lwphdc7gdf6n0y4vaaa0x7ck778kg638lk0nqv2yd343qda78sf69t6r");
    let public_key = PublicKey.fromBech32("npub14rnkcwkw0q5lnmjye7ffxvy7yxscyjl3u4mrr5qxsks76zctmz3qvuftjz");
    let event = EventBuilder.reactionExtended(event_id, public_key, 1, "🧡").toEvent(keys);

    // Send custom event to all relays
    await client.sendEvent(event);

    // Send custom event to a specific previously added relay
    // await client.sendEventTo(["wss://relay.damus.io"], event);

    let builder = new EventBuilder(new Kind(1111), "My custom event signer with the NostrSigner", []);
    await client.sendEventBuilder(builder);
}

main();
