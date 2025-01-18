from nostr_sdk import Filter, Alphabet, Keys, SingleLetterTag, Kind, KindStandard

keys = Keys.generate()

f = (Filter()
     .pubkey(keys.public_key())
     .kinds([Kind(0), Kind.from_std(KindStandard.TEXT_NOTE)])
     .custom_tag(SingleLetterTag.lowercase(Alphabet.J), ["test"])
     )
print(f.as_json())

f = f.kind(Kind(4)).custom_tag(SingleLetterTag.lowercase(Alphabet.J), ["append-new"])
print(f.as_json())
