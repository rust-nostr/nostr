import asyncio

from nostr_sdk import init_logger, LogLevel, NostrWalletConnectUri, Nwc


async def main():
    # Init logger
    init_logger(LogLevel.INFO)

    # Parse NWC uri
    uri = NostrWalletConnectUri.parse("nostr+walletconnect://..")

    # Initialize NWC client
    nwc = Nwc(uri)

    info = await nwc.get_info()
    print(info)

    balance = await nwc.get_balance()
    print(f"Balance: {balance} SAT")


if __name__ == '__main__':
    asyncio.run(main())
