//! web-sys WebSocket backend.

use crate::error::WebsocketResult;

use web_sys::{MessageEvent, Event, CloseEvent};
use wasm_bindgen::{JsValue, JsCast};
use wasm_bindgen::closure::Closure;
use js_sys::Function;
use wasm_bindgen_futures::{JsFuture, spawn_local};
use crate::Message;

impl From<MessageEvent> for Message {
    fn from(event: MessageEvent) -> Self {
        if let Ok(text) = event.data().dyn_into::<js_sys::JsString>() {
            Message::Text(text.into())
        } else if let Ok(binary) = event.data().dyn_into::<js_sys::ArrayBuffer>() {
            Message::Binary(js_sys::Uint8Array::new(&binary).to_vec())
        } else {
            unimplemented!("Some message types aren't implemented yet.")
        }
    }
}

/// Stream-based WebSocket.
#[derive(Debug)]
pub struct WebSocket {}


/// WebSocket sender. Also responsible for closing the connection.
#[derive(Debug, Clone)]
pub struct WebSocketSender {
    websocket: web_sys::WebSocket,
}

/// WebSocket receiver.
#[derive(Debug)]
pub struct WebSocketReceiver {
    receiver: async_channel::Receiver<WebsocketResult<Message>>,
    _on_message_callback: Closure<dyn FnMut(MessageEvent)>,
    _on_close_callback: Closure<dyn FnMut(CloseEvent)>
}

impl WebSocket {
    /// Creates a new WebSocket and connects it to the specified `url`.
    /// Returns `ConnectionError` if it can't connect.
    pub async fn new(url: &str) -> WebsocketResult<(WebSocketSender, WebSocketReceiver)> {
        let (sender, receiver) = async_channel::unbounded();

        let mut connection_callback = Box::new(|accept: Function, reject: Function| {
            // Connection
            let websocket = web_sys::WebSocket::new(url).expect("Couldn't create WebSocket.");
            {
                let js_value = websocket.clone().dyn_into::<JsValue>().expect("Couldn't cast WebSocket to JsValue.");
                let onopen_callback = Closure::wrap(Box::new(move |_event| {
                    accept.call1(&JsValue::NULL, &js_value).ok();
                }) as Box<dyn FnMut(Event)>);
                websocket.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
                onopen_callback.forget();
            }

            // Error handling.
            let onerror_callback = Closure::wrap(Box::new(move |_event| {
                reject.call0(&JsValue::NULL).ok();
            }) as Box<dyn FnMut(Event)>);
            websocket.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
            onerror_callback.forget();
        }) as Box<dyn FnMut(Function, Function)>;
        let connection_promise = js_sys::Promise::new(&mut connection_callback);

        JsFuture::from(connection_promise).await.map(move |websocket| {
            let websocket: web_sys::WebSocket = websocket.dyn_into().expect("Couldn't cast JsValue to WebSocket.");

            // Message streaming.
            let _on_message_callback = {
                let sender = sender.clone();
                let _on_message_callback = Closure::wrap(Box::new(move |e: MessageEvent| {
                    let sender = sender.clone();
                    spawn_local(async move {
                        sender.send(Ok(e.into())).await.ok();
                    })
                }) as Box<dyn FnMut(MessageEvent)>);
                websocket.set_onmessage(Some(_on_message_callback.as_ref().unchecked_ref()));
                _on_message_callback
            };
            // Close event.
            let _on_close_callback = Closure::wrap(Box::new(move |_e: CloseEvent| {
                sender.close();
            }) as Box<dyn FnMut(CloseEvent)>);
            websocket.set_onclose(Some(_on_close_callback.as_ref().unchecked_ref()));

            websocket.set_binary_type(web_sys::BinaryType::Arraybuffer);
            (
                WebSocketSender { websocket },
                WebSocketReceiver { receiver, _on_message_callback, _on_close_callback }
            )
        }).map_err(|error| crate::error::Error::ConnectionError(error))
    }
}

impl WebSocketSender {
    /// Attempts to close the connection and returns `SendError` if it fails.
    pub async fn close(&mut self, message: Option<String>) -> WebsocketResult<()> {
        self.send(&Message::Close(message)).await
    }

    /// Attempts to send a message and returns `SendError` if it fails.
    pub async fn send(&mut self, message: &Message) -> WebsocketResult<()> {
        if self.websocket.ready_state() == web_sys::WebSocket::OPEN {
            match message {
                Message::Text(text) => {
                    self.websocket.send_with_str(&text)
                        .map_err(|error| crate::error::Error::SendError(error))
                },
                Message::Binary(binary) => {
                    self.websocket.send_with_u8_array(&binary)
                        .map_err(|error| crate::error::Error::SendError(error))
                },
                Message::Close(reason) => {
                    const NORMAL: u16 = 1000;
                    if let Some(reason) = reason {
                        self.websocket.close_with_code_and_reason(NORMAL, &reason)
                    } else {
                        self.websocket.close_with_code(NORMAL)
                    }.map_err(|error| crate::error::Error::SendError(error))
                }
            }
        } else {
            Err(crate::error::Error::SendError("Sending while the socket is not open is not allowed.".into()))
        }
    }
}

impl WebSocketReceiver {
    /// Attempts to receive a message and returns `ReceiveError` if it fails.
    pub async fn next(&mut self) -> Option<WebsocketResult<Message>> {
        self.receiver.recv().await.ok()
    }
}

pub type Error = JsValue;