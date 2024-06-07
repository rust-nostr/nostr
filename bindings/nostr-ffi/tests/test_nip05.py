import asyncio
from nostr_ffi import *

async def main():
    nip_05 = "yuki@yukikishimoto.com"
    public_key = PublicKey.parse("npub1drvpzev3syqt0kjrls50050uzf25gehpz9vgdw08hvex7e0vgfeq0eseet")
    proxy = None
    if await verify_nip05(public_key, nip_05, proxy):
        print(f"     '{nip_05}' verified, for {public_key.to_bech32()}")
    else:
        print(f"     Unable to verify NIP-05, for {public_key.to_bech32()}")

if __name__ == '__main__':
    asyncio.run(main())