//! web-sys WebSocket backend.

use crate::error::WebsocketResult;

pub type Message = JsString;

use web_sys::{MessageEvent, Event};
use wasm_bindgen::{JsValue, JsCast};
use wasm_bindgen::closure::Closure;
use js_sys::{Function, JsString};
use wasm_bindgen_futures::{JsFuture, spawn_local};

/// Stream based WebSocket.
#[derive(Debug)]
pub struct WebSocket {
    websocket: web_sys::WebSocket,
    receiver: async_channel::Receiver<WebsocketResult<Message>>,
    _on_message_callback: Closure<dyn FnMut(MessageEvent)>
}

impl WebSocket {
    /// Creates a new WebSocket and connects it to the specified `url`.
    /// Returns `ConnectionError` if it can't connect.
    pub async fn new(url: &str) -> WebsocketResult<Self> {
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
            let _on_message_callback = Closure::wrap(Box::new(move |e: MessageEvent| {
                if let Ok(text) = e.data().dyn_into::<js_sys::JsString>() {
                    let sender = sender.clone();
                    spawn_local(async move {
                        sender.send(Ok(text)).await.ok();
                    })
                }
            }) as Box<dyn FnMut(MessageEvent)>);
            websocket.set_onmessage(Some(_on_message_callback.as_ref().unchecked_ref()));
            WebSocket { websocket, receiver, _on_message_callback }
        }).map_err(|error| crate::error::Error::ConnectionError(error))
    }

    /// Attempts to send a message and returns `SendError` if it fails.
    pub async fn send(&mut self, message: &str) -> WebsocketResult<()> {
        self.websocket.send_with_str(message)
            .map_err(|error| crate::error::Error::SendError(error))
    }

    /// Attempts to receive a message and returns `ReceiveError` if it fails.
    pub async fn next(&mut self) -> Option<WebsocketResult<Message>> {
        self.receiver.recv().await.ok()
    }
}

pub type Error = JsValue;