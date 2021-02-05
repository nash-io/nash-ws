#[cfg(test)]
mod tests {
    use nash_ws::{WebSocket, Message};
    use cross_test::prelude::*;

    cross_test_configure!();

    #[cross_test]
    async fn connection_failure() {
        WebSocket::new("wss://nonexistent.domain").await.expect_err("Couldn't connect");
    }

    #[cross_test]
    async fn send_after_close() {
        let (mut sender, _) = WebSocket::new("wss://echo.websocket.org").await.expect("Couldn't connect");

        sender.close(None).await.expect("Failed to close WebSocket.");

        let expected_message_1 = Message::Text("Hello".into());
        sender.send(&expected_message_1).await.expect_err("Sending after closing is not allowed.");
    }

    #[cross_test]
    async fn echo() {
        let (mut sender, mut receiver) = WebSocket::new("wss://echo.websocket.org").await.expect("Couldn't connect");

        let expected_message_1 = Message::Text("Hello".into());
        sender.send(&expected_message_1).await.expect("Couldn't send text message 1.");

        let expected_message_2 = Message::Binary(vec![1, 2, 3]);
        sender.send(&expected_message_2).await.expect("Couldn't send text message 2.");

        let expected_message_3 = Message::Text("Echo".into());
        sender.send(&expected_message_3).await.expect("Couldn't send binary message 3.");

        let message = receiver.next().await.expect("Stream has terminated at message 1.").expect("Failed to receive message 1.");
        assert_eq!(message, expected_message_1, "Unexpected message 1.");

        let message = receiver.next().await.expect("Stream has terminated at message 2.").expect("Failed to receive message 2.");
        assert_eq!(message, expected_message_2, "Unexpected message 2.");

        let message = receiver.next().await.expect("Stream has terminated at message 3.").expect("Failed to receive message 3.");
        assert_eq!(message, expected_message_3, "Unexpected message 3.");
    }
}
