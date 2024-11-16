from nostr_sdk import *

addr = "30023:aa4fc8665f5696e33db7e1a572e3b0f5b3d615837b0f362dcb1c8068b098c7b4:ipsum"
coordinate = Coordinate.parse(addr)
assert addr == coordinate.__str__()

t_kind = TagKind.NONCE()
assert tag_kind_to_string(t_kind) == "nonce"
