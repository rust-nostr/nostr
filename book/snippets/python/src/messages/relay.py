from nostr_sdk import RelayMessage, RelayMessageEnum, EventBuilder, Keys


def relay_message():

    keys = Keys.generate()
    event = EventBuilder.text_note("TestTextNoTe",[]).sign_with_keys(keys)

    print()
    print("Relay Messages:")

    # ANCHOR: event-message
    # Create Event relay message
    print("  Event Relay Message:")
    relay_message = RelayMessage.event("subscription_ID_abc123", event)
    print(f"     - Event Message: {relay_message.as_enum().is_event_msg()}")
    print(f"     - JSON: {relay_message.as_json()}")
    # ANCHOR_END: event-message

    print()
    # ANCHOR: ok-message
    # Create event acceptance relay message
    print("  Event Acceptance Relay Message:")
    relay_message = RelayMessage.ok(event.id(), False, "You have no power here, Gandalf The Grey")
    print(f"     - Event Acceptance Message: {relay_message.as_enum().is_ok()}")
    print(f"     - JSON: {relay_message.as_json()}")
    # ANCHOR_END: ok-message

    print()
    # ANCHOR: eose-message
    # Create End of Stored Events relay message
    print("  End of Stored Events Relay Message:")
    relay_message = RelayMessage.eose("subscription_ID_abc123")
    print(f"     - End of Stored Events Message: {relay_message.as_enum().is_end_of_stored_events()}")
    print(f"     - JSON: {relay_message.as_json()}")
    # ANCHOR_END: eose-message

    print()
    # ANCHOR: closed-message
    # Create Closed relay message
    print("  Closed Relay Message:")
    relay_message = RelayMessage.closed("subscription_ID_abc123", "So long and thanks for all the fish")
    print(f"     - Closed Message: {relay_message.as_enum().is_closed()}")
    print(f"     - JSON: {relay_message.as_json()}")
    # ANCHOR_END: closed-message

    print()
    # ANCHOR: notice-message
    # Create Notice relay message
    print("  Notice Relay Message:")
    relay_message = RelayMessage.notice("You have been served")
    print(f"     - Notice Message: {relay_message.as_enum().is_notice()}")
    print(f"     - JSON: {relay_message.as_json()}")
    # ANCHOR_END: notice-message

    print()
    # ANCHOR: parse-message
    # Parse Messages from JSON and/or Enum
    print("  Parse Relay Messages:")
    relay_message = RelayMessage.from_json('["NOTICE","You have been served"]')
    print(f"     - ENUM: {relay_message.as_enum()}")
    relay_message = RelayMessage.from_enum(RelayMessageEnum.NOTICE("You have been served"))
    print(f"     - JSON: {relay_message.as_json()}")
    # ANCHOR_END: parse-message

    print()
    # ANCHOR: auth-message
    # Create Authorization relay message (NIP42)
    print("  Auth Relay Message:")
    relay_message = RelayMessage.auth("I Challenge You To A Duel! (or some other challenge string)")
    print(f"     - Auth Message: {relay_message.as_enum().is_auth()}")
    print(f"     - JSON: {relay_message.as_json()}")
    # ANCHOR_END: auth-message

    print()
    # ANCHOR: count-message
    # Create Count relay message (NIP45)
    print("  Count Relay Message:")
    relay_message = RelayMessage.count("subscription_ID_abc123", 42)
    print(f"     - Count Message: {relay_message.as_enum().is_count()}")
    print(f"     - JSON: {relay_message.as_json()}")
    # ANCHOR_END: count-message

    print()
    # ANCHOR: neg-code
    # Negative Error Code
    print("  Negative Relay Message (code):")
    relay_message_neg = RelayMessageEnum.NEG_ERR("subscription_ID_abc123", "404")
    relay_message = RelayMessage.from_enum(relay_message_neg)
    print(f"     - Negative Error Code: {relay_message.as_enum().is_neg_err()}")
    print(f"     - JSON: {relay_message.as_json()}")
    # ANCHOR_END: neg-code

    print()
    # ANCHOR: neg-msg
    # Negative Error Message
    print("  Negative Relay Message (message):")
    relay_message_neg = RelayMessageEnum.NEG_MSG("subscription_ID_abc123", "This is not the message you are looking for")
    relay_message = RelayMessage.from_enum(relay_message_neg)
    print(f"     - Negative Error Message: {relay_message.as_enum().is_neg_msg()}")
    print(f"     - JSON: {relay_message.as_json()}")
    # ANCHOR_END: neg-msg
