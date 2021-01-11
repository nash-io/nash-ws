//! nash-ws error module.

/// Specific error to each backend.
pub type BackendError = crate::backend::Error;

/// All possible errors emitted by the WebSocket.
#[derive(Debug)]
pub enum Error {
    /// Error emitted if the connection fails.
    ConnectionError(BackendError),
    /// Error emitted if sending a message fails.
    SendError(BackendError),
    /// Error emitted if receiving a message fails.
    ReceiveError(BackendError)
}

/// Result type of nash-ws.
pub type WebsocketResult<T> = Result<T, Error>;
