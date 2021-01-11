use crate::prelude::*;
use crate::error::WebsocketResult;
use tokio_tungstenite::{connect_async, WebSocketStream, MaybeTlsStream};
pub use tokio_tungstenite::tungstenite::Error;
use tokio::net::TcpStream;
use futures_util::SinkExt;

use tokio_tungstenite::tungstenite::Message;
use futures_util::StreamExt;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct WebSocket {
    #[derivative(Debug="ignore")]
    socket: WebSocketStream<MaybeTlsStream<TcpStream>>
}

impl WebSocket {
    pub async fn new(url: &str) -> WebsocketResult<Self> {
        connect_async(url)
            .await
            .map(|(socket, _response)| Self { socket })
            .map_err(|error| crate::error::Error::ConnectionFailure(error))
    }

    pub async fn send(&mut self, message: &str) -> WebsocketResult<()> {
        self.socket.send(message.into()).await.map_err(|error| crate::error::Error::SendError(error))
    }

    pub async fn next(&mut self) -> Option<WebsocketResult<Message>> {
        self.socket.next().await.map(|result| result.map_err(|error| crate::error::Error::ReceiveError(error)))
    }
}
