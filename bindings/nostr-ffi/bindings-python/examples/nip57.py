from nostr_protocol import Keys, ZapRequestData, PublicKey, SecretKey, EventBuilder, anonymous_zap_request, private_zap_request

secret_key = SecretKey.from_hex("6b911fd37cdf5c81d4c0adb1ab7fa822ed253ab0ad9aa18d77257c88b29b718e")
keys = Keys(secret_key)

public_key = PublicKey.from_bech32("npub14f8usejl26twx0dhuxjh9cas7keav9vr0v8nvtwtrjqx3vycc76qqh9nsy")
relays = ["wss://relay.damus.io"]
msg = "Zap!"
data = ZapRequestData(public_key, relays).message(msg)

public_zap = EventBuilder.new_zap_request(data).to_event(keys)
print(f"Public zap request: {public_zap.as_json()}\n")

anon_zap = anonymous_zap_request(data)
print(f"Anonymous zap request: {anon_zap.as_json()}\n")

private_zap = private_zap_request(data, keys)
print(f"Private zap request: {private_zap.as_json()}")