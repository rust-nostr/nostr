use thiserror::Error;

#[derive(Error, Debug)]
pub enum NostrError {
    #[error("websocket Message failed to parse")]
    FailedMessageParse,
    #[error("Event failed to parse")]
    FailedEventParse
}
