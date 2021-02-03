//! Message module.

/// An enum representing the various forms of a WebSocket message.
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Message {
    /// A text WebSocket message.
    Text(String),
    /// A binary WebSocket message.
    Binary(Vec<u8>),
    /// Message sent when the connection is closed.
    Close(Option<String>)
}