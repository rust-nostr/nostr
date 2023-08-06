from nostr_sdk import Filter, Keys

keys = Keys.generate()

filter = Filter().pubkey(keys.public_key()).kinds([0, 1]).custom_tag("j", ["test"])
print(filter.as_json())

filter = filter.kind(4).custom_tag("j", ["append-new"])
print(filter.as_json())