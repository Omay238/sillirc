// based on https://github.com/snapview/tokio-tungstenite/blob/master/examples/client.rs

use sillirc_lib::networker::{Networker, SerializableMessage, SerializableMessageType};
use sillirc_lib::user::User;
use tokio::io::AsyncReadExt as _;

async fn print_message(message: SerializableMessage) {
    println!(
        "{}",
        serde_json::to_string(&message).expect("Invalid message content")
    );
}

#[tokio::main]
async fn main() {
    let user = User::new(String::from("OwOmay"));

    let mut nw = Networker::new("ws://0.0.0.0:80", print_message).await;

    let mut stdin = tokio::io::stdin();

    loop {
        let mut buf = vec![0; 1024];
        let n = match stdin.read(&mut buf).await {
            Err(_) | Ok(0) => break,
            Ok(n) => n,
        };
        buf.truncate(n);
        nw.send(SerializableMessage::new(
            user.clone(),
            SerializableMessageType::Text,
            String::from_utf8(buf)
                .expect("Failed UTF-8 unwrap")
                .to_string(),
        ))
        .await;
    }
}
