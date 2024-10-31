from nostr_sdk_ffi import *

# Test PublicKey
pk1 = PublicKey.from_hex("b2d670de53b27691c0c3400225b65c35a26d06093bcc41f48ffc71e0907f9d4a")
pk2 = PublicKey.from_hex("b2d670de53b27691c0c3400225b65c35a26d06093bcc41f48ffc71e0907f9d4a")
pk3 = PublicKey.from_hex("3bf0c63fcb93463407af97a5e5ee64fa883d107ef9e558472c4eb9aaaefa459d")

assert pk1 == pk2
assert pk1 != pk3

# Test EventId
id1 = EventId.from_hex("7469af3be8c8e06e1b50ef1caceba30392ddc0b6614507398b7d7daa4c218e96")
id2 = EventId.from_hex("7469af3be8c8e06e1b50ef1caceba30392ddc0b6614507398b7d7daa4c218e96")

assert id1 == id2

# Test Kind
k1 = Kind(1)
k2 = Kind(1)
k3 = Kind(4)

assert k1 == k2
assert k1 != k3
