from nostr_sdk import Keys, Event, ZapRequestData, PublicKey, SecretKey, EventBuilder, nip57_anonymous_zap_request, nip57_private_zap_request, nip57_decrypt_private_zap_message

secret_key = SecretKey.parse("6b911fd37cdf5c81d4c0adb1ab7fa822ed253ab0ad9aa18d77257c88b29b718e")
keys = Keys(secret_key)

public_key = PublicKey.parse("npub14f8usejl26twx0dhuxjh9cas7keav9vr0v8nvtwtrjqx3vycc76qqh9nsy")
relays = ["wss://relay.damus.io"]
msg = "Zap!"
data = ZapRequestData(public_key, relays).message(msg)

public_zap = EventBuilder.public_zap_request(data).sign_with_keys(keys)
print(f"Public zap request: {public_zap.as_json()}\n")

anon_zap = nip57_anonymous_zap_request(data)
print(f"Anonymous zap request: {anon_zap.as_json()}\n")

private_zap = nip57_private_zap_request(data, keys)
print(f"Private zap request: {private_zap.as_json()}\n")

# Decode private zap message
event_msg: Event = nip57_decrypt_private_zap_message(secret_key, public_key, private_zap)
print(f"Private zap msg: {event_msg.content()}")