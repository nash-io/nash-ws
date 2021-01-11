//! nash-ws is a web and native stream based WebSocket client.

#![warn(missing_docs)]

mod prelude;
mod backend;
pub mod error;

pub use backend::WebSocket;