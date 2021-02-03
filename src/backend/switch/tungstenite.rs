//! tungstenite WebSocket backend.

use crate::prelude::*;
use crate::error::WebsocketResult;
use tokio_tungstenite::{connect_async, WebSocketStream, MaybeTlsStream};
pub use tokio_tungstenite::tungstenite::Error;
use tokio::net::TcpStream;
use futures_util::SinkExt;

use crate::message::Message;
use futures_util::StreamExt;
use tokio_tungstenite::tungstenite::protocol::CloseFrame;
use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;

impl From<tokio_tungstenite::tungstenite::Message> for Message {
    fn from(message: tokio_tungstenite::tungstenite::Message) -> Self {
        match message {
            tokio_tungstenite::tungstenite::Message::Binary(binary) => {
                Message::Binary(binary)
            },
            tokio_tungstenite::tungstenite::Message::Text(text) => {
                Message::Text(text)
            },
            tokio_tungstenite::tungstenite::Message::Close(close) => {
                let close = close.map(|close| close.reason.to_string());
                Message::Close(close)
            },
            _ => unimplemented!("Some message types aren't implemented yet: {:#?}", message)
        }
    }
}

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
    pub async fn send(&mut self, message: &Message) -> WebsocketResult<()> {
        match message {
            Message::Text(text) => {
                self.socket.send(text.clone().into()).await.map_err(|error| crate::error::Error::SendError(error))
            },
            Message::Binary(binary) => {
                self.socket.send(binary.clone().into()).await.map_err(|error| crate::error::Error::SendError(error))
            },
            Message::Close(close) => {
                self.socket.close(close.clone().map(|reason| CloseFrame {
                    code: CloseCode::Normal,
                    reason: reason.into()
                })).await.map_err(|error| crate::error::Error::SendError(error))
            }
        }
    }

    /// Attempts to close the connection and returns `SendError` if it fails.
    pub async fn close(&mut self, message: Option<String>) -> WebsocketResult<()> {
        self.send(&Message::Close(message)).await
    }

    /// Attempts to receive a message and returns `ReceiveError` if it fails.
    pub async fn next(&mut self) -> Option<WebsocketResult<Message>> {
        self.socket.next().await.map(|result| {
            result
                .map(|result| {
                    result.into()
                })
                .map_err(|error| {
                    crate::error::Error::ReceiveError(error)
                })
        })
    }
}
