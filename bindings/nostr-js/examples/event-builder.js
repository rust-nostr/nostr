const { Keys, EventBuilder, EventId, PublicKey, Tag, loadWasmSync, Timestamp } = require("../");

function main() {
    loadWasmSync();
    
    // Generate new random keys
    let keys = Keys.generate();
    
    let event = new EventBuilder(1, "Testing nostr JS bindings", [Tag.parse(["p", "d0a59cd44b6051708e9d437aa01f86451378a130ea7ba38ad43eae0bd0e0c4ce"])]).toEvent(keys);
    console.log(event.asJson()); // Print event as JSON
    console.log(event.tags[0].toVec()); // Print first tag

    // Reaction event
    let event_id = EventId.fromBech32("note1z3lwphdc7gdf6n0y4vaaa0x7ck778kg638lk0nqv2yd343qda78sf69t6r");
    let public_key = PublicKey.fromBech32("npub14rnkcwkw0q5lnmjye7ffxvy7yxscyjl3u4mrr5qxsks76zctmz3qvuftjz");
    let reaction = EventBuilder.reaction(event_id, public_key, "🧡").toEvent(keys);
    console.log(reaction.asJson());

    // Custom created at
    let customTimestamp = Timestamp.fromSecs(12345);
    let e = EventBuilder.textNote("Event with custom timestamp", []).customCreatedAt(customTimestamp).toEvent(keys);
    console.log(e.asJson());
}

main();