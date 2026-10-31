//! Single unauthenticated handshake: no keyring, audio, session update or retry.
use std::time::Duration;
use vocalnotes_lib::platform::stt_transport;

#[tokio::main]
async fn main() {
    let request = stt_transport::request(None).expect("fixed public URL");
    let result = tokio::time::timeout(
        Duration::from_secs(15),
        tokio_tungstenite::connect_async_with_config(
            request,
            Some(
                tokio_tungstenite::tungstenite::protocol::WebSocketConfig::default()
                    .max_message_size(Some(65_536))
                    .max_frame_size(Some(65_536)),
            ),
            false,
        ),
    )
    .await;
    match result {
        Err(_) => println!("{}", stt_transport::Failure::Timeout.message()),
        Ok(Ok((socket, _))) => {
            drop(socket);
            println!("STT unauthenticated: handshake accepted; socket dropped without payload");
        }
        Ok(Err(error)) => {
            println!("{}", stt_transport::classify(&error).message());
            if let tokio_tungstenite::tungstenite::Error::Protocol(error) = &error {
                println!("Protocol variant: {}", stt_transport::protocol_code(error));
            }
        }
    }
}
