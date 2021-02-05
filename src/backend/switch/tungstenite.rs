//! tungstenite WebSocket backend.

use crate::prelude::*;
use crate::error::WebsocketResult;
use tokio_tungstenite::{connect_async, WebSocketStream, MaybeTlsStream};
pub use tokio_tungstenite::tungstenite::Error;
use tokio::net::TcpStream;
use futures_util::SinkExt;
use std::sync::{Arc, Mutex};

use crate::message::Message;
use futures_util::StreamExt;
// use tokio_tungstenite::tungstenite::protocol::CloseFrame;
// use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
use futures_util::stream::{SplitSink, SplitStream};

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

/// Stream-based WebSocket.
#[derive(Derivative)]
#[derivative(Debug)]
pub struct WebSocket {}

/// WebSocket sender. Also responsible for closing the connection.
#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct WebSocketSender {
    #[derivative(Debug="ignore")]
    sender: Arc<Mutex<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, tokio_tungstenite::tungstenite::Message>>>
}

/// Stream-based WebSocket receiver.
#[derive(Derivative)]
#[derivative(Debug)]
pub struct WebSocketReceiver {
    #[derivative(Debug="ignore")]
    receiver: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>
}

impl WebSocket {
    /// Creates a new WebSocket and connects it to the specified `url`.
    /// Returns `ConnectionError` if it can't connect.
    pub async fn new(url: &str) -> WebsocketResult<(WebSocketSender, WebSocketReceiver)> {
        connect_async(url)
            .await
            .map(|(socket, _response)| {
                let (sender, receiver) = socket.split();
                let sender = Arc::new(Mutex::new(sender));
                (WebSocketSender { sender }, WebSocketReceiver { receiver })
            })
            .map_err(|error| crate::error::Error::ConnectionError(error))
    }
}

impl WebSocketSender {
    /// Attempts to send a message and returns `SendError` if it fails.
    pub async fn send(&mut self, message: &Message) -> WebsocketResult<()> {
        match message {
            Message::Text(text) => {
                let text = text.clone().into();
                self.sender.lock().expect("Failed to lock sender.").send(text).await.map_err(|error| crate::error::Error::SendError(error))
            },
            Message::Binary(binary) => {
                let binary = binary.clone().into();
                self.sender.lock().expect("Failed to lock sender.").send(binary).await.map_err(|error| crate::error::Error::SendError(error))
            },
            Message::Close(_close) => {
                self.sender.lock().expect("Failed to lock sender.").close().await.map_err(|error| crate::error::Error::SendError(error))
            }
        }
    }

    /// Attempts to close the connection and returns `SendError` if it fails.
    pub async fn close(&mut self, message: Option<String>) -> WebsocketResult<()> {
        self.send(&Message::Close(message)).await
    }
}

impl WebSocketReceiver {
    /// Attempts to receive a message and returns `ReceiveError` if it fails.
    pub async fn next(&mut self) -> Option<WebsocketResult<Message>> {
        self.receiver.next().await.map(|result| {
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
