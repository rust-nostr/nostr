import asyncio
from nostr_sdk import Client, MockRelay


# TODO: move to Rust Nostr Book

async def main():
    # Run mock relay
    mock: MockRelay = await MockRelay.run()
    mock_relay_url = mock.url()

    client = Client()

    await client.add_relay(mock_relay_url)
    await client.connect()

    # ...


if __name__ == '__main__':
    asyncio.run(main())
