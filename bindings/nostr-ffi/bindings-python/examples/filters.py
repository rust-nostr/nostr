from nostr_protocol import Filter, Alphabet, Keys, SingleLetterTag, Kind, KindEnum

keys = Keys.generate()

filter = Filter().pubkey(keys.public_key()).kinds([Kind(0), Kind.from_enum(KindEnum.TEXT_NOTE())]).custom_tag(SingleLetterTag.lowercase(Alphabet.J), ["test"])
print(filter.as_json())

filter = filter.kind(Kind(4)).custom_tag(SingleLetterTag.lowercase(Alphabet.J), ["append-new"])
print(filter.as_json())