package rust.nostr.snippets

import rust.nostr.sdk.*

// ANCHOR: json
fun json() {
    // Deserialize from json
    val json =
        "{\"content\":\"uRuvYr585B80L6rSJiHocw==?iv=oh6LVqdsYYol3JfFnXTbPA==\",\"created_at\":1640839235,\"id\":\"2be17aa3031bdcb006f0fce80c146dea9c1c0268b0af2398bb673365c6444d45\",\"kind\":4,\"pubkey\":\"f86c44a2de95d9149b51c6a29afeabba264c18e2fa7c49de93424a0c56947785\",\"sig\":\"a5d9290ef9659083c490b303eb7ee41356d8778ff19f2f91776c8dc4443388a64ffcf336e61af4c25c05ac3ae952d1ced889ed655b67790891222aaa15b99fdd\",\"tags\":[[\"p\",\"13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d\"]]}"
    val event = Event.fromJson(json)

    // Serialize as json
    println(event.asJson())
}
// ANCHOR_END: json

// ANCHOR: builder
fun builder() {
    val keys = Keys.generate();

    // Compose custom event
    val customEvent = EventBuilder(Kind(1111u), "", listOf()).signWithKeys(keys);

    // Compose text note
    val textNoteEvent = EventBuilder.textNote("Hello", listOf()).signWithKeys(keys);

    // Compose reply to above text note
    val replyEvent = EventBuilder.textNote("Reply to hello", listOf(Tag.event(textNoteEvent.id())))
        .signWithKeys(keys);

    // Compose POW event
    val powEvent =
    EventBuilder.textNote("Another reply with POW", listOf(Tag.event(textNoteEvent.id())))
        .pow(20u)
        .signWithKeys(keys);
    println(powEvent.asJson())
}
// ANCHOR_END: builder
