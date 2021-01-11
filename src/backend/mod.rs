#[cfg(not(target_arch = "wasm32"))]
mod switch {
    mod tungstenite;
    pub use tungstenite::*;
}

#[cfg(target_arch = "wasm32")]
mod switch {
    mod web_sys;
    pub use self::web_sys::*;
}

pub use switch::*;
