use nostr::nips::nip65;
use nostr::prelude::FromBech32;
use nostr::secp256k1::XOnlyPublicKey;
use nostr::{ClientMessage, Filter, Kind, RelayMessage, Result, SubscriptionId};
use tungstenite::{connect, Message as WsMessage};

fn main() -> Result<()> {
    env_logger::init();

    let public_key = XOnlyPublicKey::from_bech32(
        "npub1acg6thl5psv62405rljzkj8spesceyfz2c32udakc2ak0dmvfeyse9p35c",
    )?;

    let (mut socket, _response) =
        connect("wss://nostr.mikedilger.com").expect("Can't connect to relay");

    let subscribe_msg = ClientMessage::new_req(
        SubscriptionId::generate(),
        vec![Filter::new()
            .author(public_key.to_string())
            .kind(Kind::RelayList)],
    );

    println!("Subscribing to Relay List Metadata");
    socket.write_message(WsMessage::Text(subscribe_msg.as_json()))?;

    let msg = socket.read_message().expect("Error reading message");
    let msg_text = msg.to_text().expect("Failed to convert message to text");
    if let Ok(RelayMessage::Event { event, .. }) = RelayMessage::from_json(msg_text) {
        if event.kind == Kind::RelayList {
            let list = nip65::extract_relay_list(&*event);
            println!("Found relay list metadata: {list:?}");
        }
    }

    Ok(())
}
