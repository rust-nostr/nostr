const { Keys, EventBuilder } = require("../");

function main() {
    // Generate new random keys
    let keys = Keys.generate();
    
    let event = new EventBuilder(BigInt(1), "Testing nostr JS bindings", [["p", "d0a59cd44b6051708e9d437aa01f86451378a130ea7ba38ad43eae0bd0e0c4ce"]]).toEvent(keys);
    console.log(event.asJson());
}

main();