from nostr_protocol import Filter, Alphabet, Keys, SingleLetterTag

keys = Keys.generate()

filter = Filter().pubkey(keys.public_key()).kinds([0, 1]).custom_tag(SingleLetterTag.lowercase(Alphabet.J), ["test"])
print(filter.as_json())

filter = filter.kind(4).custom_tag(SingleLetterTag.lowercase(Alphabet.J), ["append-new"])
print(filter.as_json())