const { Keys, EventBuilder, Kind, Tag, loadWasmSync, Timestamp } = require("../");

function main() {
    loadWasmSync();
    
    // Generate new random keys
    let keys = Keys.generate();

    let kind = new Kind(1);
    let event = new EventBuilder(kind, "Testing nostr JS bindings", [Tag.parse(["p", "d0a59cd44b6051708e9d437aa01f86451378a130ea7ba38ad43eae0bd0e0c4ce"])]).toEvent(keys);
    console.log(event.asJson()); // Print event as JSON
    console.log(event.tags[0].toVec()); // Print first tag

    // Custom created at
    let customTimestamp = Timestamp.fromSecs(12345);
    let e = EventBuilder.textNote("Event with custom timestamp", []).customCreatedAt(customTimestamp).toEvent(keys);
    console.log(e.asJson());
}

main();