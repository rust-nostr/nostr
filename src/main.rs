use tungstenite::{connect, Message};
use url::Url;

fn main() {
    env_logger::init();

    let (mut socket, response) =
        connect(Url::parse("wss://nostr-relay.herokuapp.com/ws").unwrap()).expect("Can't connect");

    println!("Connected to the server");
    println!("Response HTTP code: {}", response.status());
    println!("Response contains the following headers:");
    for (ref header, _value) in response.headers() {
        println!("* {}", header);
    }

    socket.write_message(Message::Text("sub-key:379e863e8357163b5bce5d2688dc4f1dcc2d505222fb8d74db600f30535dfdfe".into())).unwrap();
    loop {
        let msg = socket.read_message().expect("Error reading message");
        println!("Received: {}", msg);
        match msg.to_text() {
            Ok(msg) => {
                println!("{}", msg);
                if msg == "PING" {
                    println!("Got PING, sending PONG");
                    socket.write_message(Message::Text("PONG".into())).unwrap();
                }
            },
            Err(e) => {
                eprintln!("{}", e);
            }


        }
        
    }
    // socket.close(None);
}
