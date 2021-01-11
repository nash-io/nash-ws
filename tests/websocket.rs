mod cross {
    #[cfg(not(target_arch = "wasm32"))]
    mod platform {
        pub use tokio::test as test;
    }

    #[cfg(target_arch = "wasm32")]
    mod platform {
        pub use wasm_bindgen_test::wasm_bindgen_test_configure as web_configure;
        pub use wasm_bindgen_test::wasm_bindgen_test as test;
        web_configure!(run_in_browser);
    }

    pub use platform::*;
}

#[cfg(test)]
mod tests {
    use nash_ws::WebSocket;
    use crate::cross;

    #[cross::test]
    async fn connection_failure() {
        WebSocket::new("wss://nonexistent.domain").await.expect_err("Couldn't connect");
    }

    #[cross::test]
    async fn echo() {
        let mut websocket = WebSocket::new("wss://echo.websocket.org").await.expect("Couldn't connect");

        let expected_message_1 = "Hello";
        websocket.send(expected_message_1.into()).await.expect("Couldn't send message.");
        let expected_message_2 = "Echo";
        websocket.send(expected_message_2.into()).await.expect("Couldn't send message.");
        let message = websocket.next().await.expect("Stream has terminated at message 1.").expect("Failed to receive message.");
        assert_eq!(message.to_string(), expected_message_1, "Unexpected message 1.");
        let message = websocket.next().await.expect("Stream has terminated at message 2.").expect("Failed to receive message.");
        assert_eq!(message.to_string(), expected_message_2, "Unexpected message 2.");
    }
}
