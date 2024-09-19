const { Keys, loadWasmSync, EventBuilder, Tag, Timestamp, Kind } = require("@rust-nostr/nostr");

function eventBuilder() {
    // Load WASM
    loadWasmSync();

    let keys = Keys.generate();

    // Compose custom event
    let kind = new Kind(1111);
    let customEvent = new EventBuilder(kind, "", []).toEvent(keys);

    // Compose text note
    let textnoteEvent = EventBuilder.textNote("Hello", []).toEvent(keys);

    // Compose reply to above text note
    let replyEvent =
        EventBuilder.textNote("Reply to hello", [Tag.event(textnoteEvent.id)])
            .toEvent(keys);

    // Compose POW event
    let powEvent =
        EventBuilder.textNote("Another reply with POW", [Tag.event(textnoteEvent.id)])
            .pow(20)
            .toEvent(keys);

    // Compose note with custom timestamp
    let customTimestamp =
        EventBuilder.textNote("Note with custom timestamp", [])
            .customCreatedAt(Timestamp.fromSecs(12345678))
            .toEvent(keys);
}

module.exports.eventBuilder = eventBuilder;
