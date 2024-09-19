from nostr_protocol import *

def event_builder():
    keys = Keys.generate()

    # Compose custom event
    custom_event = EventBuilder(Kind(1111), "", []).to_event(keys)

    # Compose text note
    textnote_event = EventBuilder.text_note("Hello", []).to_event(keys)

    # Compose reply to above text note
    reply_event = EventBuilder.text_note("Reply to hello", [Tag.event(textnote_event.id())]).to_event(keys)

    # Compose POW event
    pow_event = EventBuilder.text_note("Another reply with POW", [Tag.event(textnote_event.id())]).pow(20).to_event(keys)
