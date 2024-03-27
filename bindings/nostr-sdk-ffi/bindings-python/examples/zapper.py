import asyncio
from nostr_sdk import Keys, ClientBuilder, NostrSigner, NostrZapper, NostrWalletConnectUri, PublicKey, ZapEntity, \
    init_logger, LogLevel


async def main():
    # Init logger
    init_logger(LogLevel.INFO)

    # Parse NWC uri
    uri = NostrWalletConnectUri.parse("nostr+walletconnect://..")

    # Compose client
    keys = Keys.generate()
    signer = NostrSigner.keys(keys)
    zapper = NostrZapper.nwc(uri)
    client = ClientBuilder().signer(signer).zapper(zapper).build()

    await client.add_relay("wss://relay.damus.io")
    await client.connect()

    pk = PublicKey.from_bech32("npub1drvpzev3syqt0kjrls50050uzf25gehpz9vgdw08hvex7e0vgfeq0eseet")
    await client.zap(ZapEntity.public_key(pk), 1000, None)


if __name__ == '__main__':
    asyncio.run(main())
