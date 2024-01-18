const { Keys, loadWasmSync, EventBuilder, Tag } = require("@rust-nostr/nostr");

function eventBuilder() {
    // Load WASM
    loadWasmSync();

    let keys = Keys.generate();

    // Compose custom event
    let customEvent = new EventBuilder(1111, "", []).toEvent(keys);

    // Compose text note
    let textnoteEvent = EventBuilder.textNote("Hello", []).toEvent(keys);

    // Compose reply to above text note
    let replyEvent =
        EventBuilder.textNote("Reply to hello", [Tag.parse(["e", textnoteEvent.id.toHex()])])
            .toEvent(keys);

    // Compose POW event
    let powEvent =
        EventBuilder.textNote("Another reply with POW", [Tag.parse(["e", textnoteEvent.id.toHex()])])
            .toPowEvent(keys, 20);
}

module.exports.eventBuilder = eventBuilder;