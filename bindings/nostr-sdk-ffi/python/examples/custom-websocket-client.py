import asyncio

from datetime import timedelta

from aiohttp import ClientSession, ClientWebSocketResponse, WSMsgType
from nostr_sdk import *


class Sink(WebSocketSink):
    def __init__(self, ws: ClientWebSocketResponse):
        self.websocket = ws
        self.running = True  # Track if the connection is active

    async def send_msg(self, msg: WebSocketMessage):
        try:
            # Send the message if the WebSocket is still open
            if msg.is_text():
                await self.websocket.send_str(msg[0])
            elif msg.is_binary():
                await self.websocket.send_bytes(msg[0])
        except Exception as e:
            # Handle clean closure gracefully
            print(f"Attempted to send on a closed WebSocket: {e}")
            self.running = False  # Mark the connection as closed
            raise e

    async def terminate(self):
        # Close the WebSocket connection and update the status
        self.running = False
        try:
            await self.websocket.close()
        except Exception as e:
            raise e


class Stream:
    def __init__(self, ws: ClientWebSocketResponse, forwarder: WebSocketStreamForwarder):
        self.websocket = ws
        self.forwarder = forwarder

    def run(self):
        asyncio.create_task(self.listen())

    async def listen(self):
        while True:
            try:
                # Receive message
                raw_msg = await self.websocket.receive()

                if raw_msg.type == WSMsgType.TEXT:
                    msg = WebSocketMessage.TEXT(raw_msg.data)
                elif raw_msg.type == WSMsgType.BINARY:
                    msg = WebSocketMessage.BINARY(raw_msg.data)
                elif raw_msg.type == WSMsgType.PING:
                    msg = WebSocketMessage.PING(raw_msg.data)
                elif raw_msg.type == WSMsgType.PONG:
                    msg = WebSocketMessage.PONG(raw_msg.data)
                else:
                    continue

                if msg is not None:
                    await self.forwarder.forward(msg)
            except Exception as e:
                print(e)

class MyWebSocketClient(CustomWebSocketTransport):
    def __init__(self):
        self.session = ClientSession()

    def support_ping(self) -> bool:
        return False

    async def connect(self, url: "str", mode: "ConnectionMode", timeout) -> WebSocketAdaptor:
        try:
            ws = await self.session.ws_connect(url)

            sink = Sink(ws)
            stream = WebSocketStreamForwarder()
            adaptor = WebSocketAdaptor(sink, stream)

            stream = Stream(ws, stream)
            stream.run()

            return adaptor
        except Exception as e:
            print("connection error")
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
