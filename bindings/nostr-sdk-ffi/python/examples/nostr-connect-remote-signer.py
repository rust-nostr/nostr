import asyncio
from nostr_sdk import NostrConnectRemoteSigner, NostrConnectSignerActions, Nip46Request, init_logger, LogLevel, \
    SecretKey


async def main():
    init_logger(LogLevel.DEBUG)

    secret_key = SecretKey.parse("nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85")

    # Compose signer
    signer = await NostrConnectRemoteSigner.init(secret_key, ["wss://relay.nsec.app"])

    # Compose signer from URI
    #uri = NostrConnectUri.parse(
    #    "nostrconnect://aa4fc8665f5696e33db7e1a572e3b0f5b3d615837b0f362dcb1c8068b098c7b4?metadata=%7B%22name%22%3A%22Test+app%22%7D&relay=wss%3A%2F%2Frelay.nsec.app")
    #signer = NostrConnectRemoteSigner.from_uri(uri, secret_key)

    # Print bunker URI
    bunker_uri = await signer.bunker_uri()
    print(f"Bunker URI: {bunker_uri.__str__()}")

    # Define signer actions
    class SignerActions(NostrConnectSignerActions):
        def approve(self, req: Nip46Request) -> bool:
            # Check request
            # Return true to approve it otherwise false
            print(req)
            return True

    # Serve signer
    await signer.serve(SignerActions())


if __name__ == '__main__':
    asyncio.run(main())
