use crate::error::WebsocketResult;
use derivative::Derivative;

pub type Message = JsString;

use web_sys::{ErrorEvent, MessageEvent, Event};
use wasm_bindgen::{JsValue, JsCast};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::prelude::*;
use js_sys::{Function, JsString};
use wasm_bindgen_futures::JsFuture;
use std::collections::VecDeque;
use std::rc::Rc;
use std::cell::RefCell;
use std::future::Future;

// TODO: Make this public in a better module.
#[derive(Clone, Derivative)]
#[derivative(Debug)]
struct JsStream {
    #[derivative(Debug="ignore")]
    queue: Rc<RefCell<VecDeque<JsFuture>>>
}

impl JsStream {
    pub fn new() -> Self {
        Self { queue: Default::default() }
    }

    pub fn push(&self, item: JsFuture) {
        self.queue.borrow_mut().push_back(item)
    }

    pub async fn next(&self) -> Option<WebsocketResult<Message>> {
        let future = self.queue.borrow_mut().pop_front();
        if let Some(future) = future {
            let result = future.await
                .map(|message| {
                   message.dyn_into().expect("Couldn't convert message to JsString")
                })
                .map_err(|error| {
                    crate::error::Error::ReceiveError(error)
                });
            Some(result)
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct WebSocket {
    websocket: web_sys::WebSocket,
    stream: JsStream
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

impl WebSocket {
    pub async fn new(url: &str) -> WebsocketResult<Self> {
        let stream = JsStream::new();
        let stream_clone = stream.clone();
        let mut connection_callback = Box::new(|accept: Function, reject: Function| {

            // Connection
            let websocket = web_sys::WebSocket::new(url).expect("Couldn't create WebSocket.");
            {
                let js_value = websocket.clone().dyn_into::<JsValue>().expect("Couldn't cast WebSocket to JsValue.");
                let onopen_callback = Closure::wrap(Box::new(move |event| {
                    accept.call1(&JsValue::NULL, &js_value);
                }) as Box<dyn FnMut(Event)>);
                websocket.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
                onopen_callback.forget();
            }

            // Error handling.
            let onerror_callback = Closure::wrap(Box::new(move |event| {
                reject.call0(&JsValue::NULL);
            }) as Box<dyn FnMut(Event)>);
            websocket.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
            onerror_callback.forget();

            // Message streaming.
            let mut accept_function: Rc<RefCell<Option<Function>>> = Rc::new(RefCell::new(None));
            {
                let accept_function = accept_function.clone();
                let stream = stream_clone.clone();
                let onmessage_callback = Closure::wrap(Box::new(move |e: MessageEvent| {
                    if let Ok(text) = e.data().dyn_into::<js_sys::JsString>() {
                        accept_function.borrow()
                            .as_ref()
                            .map(|accept|
                                accept.call1(&JsValue::NULL, &text)
                            );
                        let mut message_promise_callback = Box::new(|accept: Function, reject: Function| {
                            *accept_function.borrow_mut() = Some(accept);
                        }) as Box<dyn FnMut(Function, Function)>;
                        let message_promise = js_sys::Promise::new(&mut message_promise_callback);
                        let message_future = JsFuture::from(message_promise);
                        stream.push(message_future);
                    }
                }) as Box<dyn FnMut(MessageEvent)>);
                websocket.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
                // FIXME: Make it part of WebSocket structure.
                onmessage_callback.forget();
            }

            let mut message_promise_callback = Box::new(|accept: Function, reject: Function| {
                *accept_function.borrow_mut() = Some(accept);
            }) as Box<dyn FnMut(Function, Function)>;

            let message_promise = js_sys::Promise::new(&mut message_promise_callback);
            let message_future = JsFuture::from(message_promise);
            stream_clone.push(message_future);
        }) as Box<dyn FnMut(Function, Function)>;
        let connection_promise = js_sys::Promise::new(&mut connection_callback);

        JsFuture::from(connection_promise).await.map(move |websocket| {
            let websocket = websocket.dyn_into().expect("Couldn't cast JsValue to WebSocket.");
            WebSocket { websocket, stream }
        }).map_err(|error| crate::error::Error::ConnectionFailure(error))
    }

    pub async fn send(&mut self, message: &str) -> WebsocketResult<()> {
        self.websocket.send_with_str(message)
            .map_err(|error| crate::error::Error::SendError(error))
    }

    pub async fn next(&mut self) -> Option<WebsocketResult<Message>> {
        self.stream.next().await
    }
}

pub type Error = JsValue;