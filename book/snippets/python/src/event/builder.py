from nostr_sdk import Keys, EventBuilder, Kind, Tag


def event_builder():
    keys = Keys.generate()

    # Compose custom event
    custom_event = EventBuilder(Kind(1111), "").sign_with_keys(keys)

    # Compose text note
    textnote_event = EventBuilder.text_note("Hello").sign_with_keys(keys)

    # Compose reply to above text note
    reply_event = EventBuilder.text_note("Reply to hello").tags([Tag.event(textnote_event.id())]).sign_with_keys(keys)

    # Compose POW event
    pow_event = EventBuilder.text_note("Another reply with POW").tags([Tag.event(textnote_event.id())]).pow(20).sign_with_keys(keys)
