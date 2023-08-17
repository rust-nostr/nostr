from nostr_protocol import Filter, Alphabet, Keys, Kind, KindEnum

keys = Keys.generate()

filter = Filter().pubkey(keys.public_key()).kinds([Kind(0), Kind.from_enum(KindEnum.TEXT_NOTE())]).custom_tag(Alphabet.J, ["test"])
print(filter.as_json())

filter = filter.kind(Kind.from_enum(KindEnum.ENCRYPTED_DIRECT_MESSAGE())).custom_tag(Alphabet.J, ["append-new"])
print(filter.as_json())