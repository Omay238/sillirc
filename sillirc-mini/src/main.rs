// based on https://github.com/snapview/tokio-tungstenite/blob/master/examples/client.rs

use colored::Colorize as _;
use sillirc_lib::networker::{Networker, SerializableMessage, SerializableMessageType};
use sillirc_lib::user::User;
use std::env;
use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};

static mut USER: User = User::new_static();

async fn print_message(message: SerializableMessage) {
    let content = message.get_content();
    let message_user = message.get_user();
    let (r, g, b) = message_user.get_color();

    let user;
    #[expect(unsafe_code, static_mut_refs)]
    // SAFETY:
    // Yeah I could avoid doing this but that's lame
    unsafe {
        user = USER.clone();
    }

    if user.get_username() == message_user.get_username()
        && user.get_color() == message_user.get_color()
    {
        return;
    }

    match message.get_message_type() {
        SerializableMessageType::Join => {
            println!(
                "{} has joined the chat.",
                message_user.get_username().truecolor(r, g, b)
            );
        }
        SerializableMessageType::Leave => {
            println!(
                "{} has left the chat",
                message_user.get_username().truecolor(r, g, b)
            );
        }
        SerializableMessageType::Rename => {
            println!(
                "{} changed their name to {}",
                message_user.get_username().truecolor(r, g, b),
                content.truecolor(r, g, b)
            );
        }
        SerializableMessageType::Text => {
            println!(
                "{}: {}",
                message_user.get_username().truecolor(r, g, b),
                content
            );
        }
    }
}

#[tokio::main]
async fn main() {
    let mut stdin = tokio::io::stdin();
    let mut stdout = tokio::io::stdout();

    let addr = env::args()
        .nth(2)
        .unwrap_or_else(|| String::from("ws://sillirc.owomay.hackclub.app"));

    let username = env::args()
        .nth(1)
        .unwrap_or_else(|| String::from("Anonymouse"));

    let user = User::new(username);
    #[expect(unsafe_code)]
    // SAFETY:
    // Yeah I could avoid doing this but that's lame
    unsafe {
        USER = user.clone();
    }

    let mut nw = Networker::new(&addr, print_message).await;

    let (r, g, b) = user.get_color();

    loop {
        stdout
            .write_all(format!("{}: ", user.get_username().truecolor(r, g, b)).as_bytes())
            .await
            .expect("Failed to write username");
        stdout.flush().await.expect("Failed to flush stdout");

        let mut buf = vec![0; 1024];
        let n = match stdin.read(&mut buf).await {
            Err(_) | Ok(0) => break,
            Ok(n) => n,
        };
        buf.truncate(n);
        let Ok(text_content) = String::from_utf8(buf) else {
            continue;
        };
        nw.send(SerializableMessage::new(
            user.clone(),
            SerializableMessageType::Text,
            text_content.replace('\n', ""),
        ))
        .await;
    }
}
