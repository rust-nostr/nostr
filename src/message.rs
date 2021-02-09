use serde_json::Value;

use crate::Event;

#[derive(Debug, PartialEq)]
pub enum Message {
    Ping,
    Notice(String),
    Event(Event),
    Empty, // Why am I getting these?
}

impl Message {
    pub fn handle(msg: &str) -> Result<Self, Box<dyn std::error::Error>> {
        dbg!(msg);

        if msg.is_empty() {
            return Ok(Self::Empty);
        }

        // Ping
        if msg == "PING" {
            return Ok(Self::Ping);
        }

        let v: Value = serde_json::from_str(msg)?;

        // Notice
        if v[0] == "notice" {
            let notice = v[1].to_string();
            return Ok(Self::Notice(notice));
        }

        // Regular events
        let event = Event::new_from_json(v[0].to_string())?;
        let _context = v[1].clone();

        Ok(Self::Event(event))
    }
}
