from nostr_sdk import Filter, Alphabet, Keys, SingleLetterTag, Kind, KindEnum

keys = Keys.generate()

f = (Filter()
     .pubkey(keys.public_key())
     .kinds([Kind(0), Kind.from_enum(KindEnum.TEXT_NOTE())])
     .custom_tag(SingleLetterTag.lowercase(Alphabet.J), ["test"])
     )
print(f.as_json())

f = f.kind(Kind(4)).custom_tag(SingleLetterTag.lowercase(Alphabet.J), ["append-new"])
print(f.as_json())
