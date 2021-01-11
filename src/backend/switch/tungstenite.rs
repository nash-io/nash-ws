//! tungstenite WebSocket backend.

use crate::prelude::*;
use crate::error::WebsocketResult;
use tokio_tungstenite::{connect_async, WebSocketStream, MaybeTlsStream};
pub use tokio_tungstenite::tungstenite::Error;
use tokio::net::TcpStream;
use futures_util::SinkExt;

use tokio_tungstenite::tungstenite::Message;
use futures_util::StreamExt;

/// Stream based WebSocket.
#[derive(Derivative)]
#[derivative(Debug)]
pub struct WebSocket {
    #[derivative(Debug="ignore")]
    socket: WebSocketStream<MaybeTlsStream<TcpStream>>
}

impl WebSocket {
    /// Creates a new WebSocket and connects it to the specified `url`.
    /// Returns `ConnectionError` if it can't connect.
    pub async fn new(url: &str) -> WebsocketResult<Self> {
        connect_async(url)
            .await
            .map(|(socket, _response)| Self { socket })
            .map_err(|error| crate::error::Error::ConnectionError(error))
    }

    /// Attempts to send a message and returns `SendError` if it fails.
    pub async fn send(&mut self, message: &str) -> WebsocketResult<()> {
        self.socket.send(message.into()).await.map_err(|error| crate::error::Error::SendError(error))
    }

    /// Attempts to receive a message and returns `ReceiveError` if it fails.
    pub async fn next(&mut self) -> Option<WebsocketResult<Message>> {
        self.socket.next().await.map(|result| result.map_err(|error| crate::error::Error::ReceiveError(error)))
    }
}
