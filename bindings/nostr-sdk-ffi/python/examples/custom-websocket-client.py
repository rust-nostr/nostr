import asyncio

from datetime import timedelta

from aiohttp import ClientSession, ClientWebSocketResponse, WSMsgType
from nostr_sdk import *


class MyAdapter(WebSocketAdapter):
    def __init__(self, session: ClientSession, ws: ClientWebSocketResponse):
        self.session = session
        self.websocket = ws

    async def send(self, msg: WebSocketMessage):
        try:
            if msg.is_text():
                await self.websocket.send_str(msg[0])
            elif msg.is_binary():
                await self.websocket.send_bytes(msg[0])
        except Exception as e:
            # Handle clean closure gracefully
            print(f"Attempted to send on a closed WebSocket: {e}")
            raise e

    async def recv(self) -> WebSocketMessage | None:
        try:
            # Receive message
            raw_msg = await self.websocket.receive()

            if raw_msg.type == WSMsgType.TEXT:
                return WebSocketMessage.TEXT(raw_msg.data)
            elif raw_msg.type == WSMsgType.BINARY:
                return WebSocketMessage.BINARY(raw_msg.data)
            elif raw_msg.type == WSMsgType.PING:
                return WebSocketMessage.PING(raw_msg.data)
            elif raw_msg.type == WSMsgType.PONG:
                return WebSocketMessage.PONG(raw_msg.data)
            else:
                raise "unknown message type"
        except Exception as e:
            raise e

    async def close_connection(self):
        await self.websocket.close()
        await self.session.close()

class MyWebSocketClient(CustomWebSocketTransport):
    def support_ping(self) -> bool:
        return False

    async def connect(self, url: "str", mode: "ConnectionMode", timeout) -> WebSocketAdapterWrapper:
        try:
            session = ClientSession()
            ws = await session.ws_connect(url)

            adaptor = MyAdapter(session, ws)
            wrapper = WebSocketAdapterWrapper(adaptor)

            return wrapper
        except Exception as e:
            raise e


async def main():
    uniffi_set_event_loop(asyncio.get_running_loop())

    # Init logger
    init_logger(LogLevel.TRACE)

    # Initialize client without signer
    # client = Client()

    # Or, initialize with Keys signer
    keys = Keys.parse("nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85")
    signer = NostrSigner.keys(keys)

    # Or, initialize with NIP46 signer
    # app_keys = Keys.parse("..")
    # uri = NostrConnectUri.parse("bunker://.. or nostrconnect://..")
    # connect = NostrConnect(uri, app_keys, timedelta(seconds=60), None)
    # signer = NostrSigner.nostr_connect(connect)

    client = ClientBuilder().signer(signer).websocket_transport(MyWebSocketClient()).build()
    #client = ClientBuilder().signer(signer).build()

    # Add relays and connect
    await client.add_relay("ws://127.0.0.1:7777")
    await client.connect()

    # Send an event using the Nostr Signer
    builder = EventBuilder.text_note("Test from rust-nostr Python bindings!")
    output = await client.send_event_builder(builder)

    print("Event sent:")
    print(f" hex:    {output.id.to_hex()}")
    print(f" bech32: {output.id.to_bech32()}")
    print(f" Successfully sent to:    {output.success}")
    print(f" Failed to send to: {output.failed}")

    await asyncio.sleep(2.0)

    # Get events from relays
    print("Getting events from relays...")
    f = Filter().authors([keys.public_key()])
    events = await client.fetch_events([f], timedelta(seconds=10))
    for event in events.to_vec():
        print(event.as_pretty_json())


if __name__ == '__main__':
    asyncio.run(main())
