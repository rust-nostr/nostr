const { Keys, EventBuilder, EventId, PublicKey, Tag, loadWasmSync } = require("../");

function main() {
    loadWasmSync();
    
    // Generate new random keys
    let keys = Keys.generate();
    
    let event = new EventBuilder(BigInt(1), "Testing nostr JS bindings", [new Tag(["p", "d0a59cd44b6051708e9d437aa01f86451378a130ea7ba38ad43eae0bd0e0c4ce"])]).toEvent(keys);
    console.log(event.asJson()); // Print event as JSON
    console.log(event.tags[0].toVec()); // Print first tag

    // Reaction event
    let event_id = EventId.fromBech32("note1z3lwphdc7gdf6n0y4vaaa0x7ck778kg638lk0nqv2yd343qda78sf69t6r");
    let public_key = PublicKey.fromBech32("npub14rnkcwkw0q5lnmjye7ffxvy7yxscyjl3u4mrr5qxsks76zctmz3qvuftjz");
    let reaction = EventBuilder.newReaction(event_id, public_key, "🧡").toEvent(keys);
    console.log(reaction.asJson());
}

main();