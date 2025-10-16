use crate::user::User;
use futures::StreamExt as _;
use futures::channel::mpsc::UnboundedSender;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub enum SerializableMessageType {
    Join = 0,
    Leave = 1,
    Text = 2,
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct SerializableMessage {
    user: User,
    message_type: SerializableMessageType,
    content: String,
}

impl SerializableMessage {
    pub fn new(user: User, message_type: SerializableMessageType, content: String) -> Self {
        Self {
            user,
            message_type,
            content,
        }
    }

    pub fn get_user(&self) -> User {
        self.user.clone()
    }

    pub fn get_message_type(&self) -> SerializableMessageType {
        self.message_type.clone()
    }

    pub fn get_content(&self) -> String {
        self.content.clone()
    }
}

#[derive(Clone)]
pub struct Networker {
    tx: UnboundedSender<Message>,
    // rx: UnboundedReceiver<Message>,
    // ws_stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
    // write: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    // read: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
}

impl Networker {
    pub async fn new<F, Fut>(url: &str, rx_callback: F) -> Self
    where
        F: Fn(SerializableMessage) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let (tx, rx) = futures::channel::mpsc::unbounded();
        let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
        let (write, read) = ws_stream.split();

        tokio::spawn(rx.map(Ok).forward(write));

        let rx_callback = std::sync::Arc::new(rx_callback);
        tokio::spawn({
            let rx_callback = rx_callback.clone();
            read.for_each(move |message| {
                let rx_callback = rx_callback.clone();
                async move {
                    let data = message
                        .expect("Failed to read message")
                        .into_text()
                        .expect("Failed to convert message to text");
                    let msg: SerializableMessage =
                        serde_json::from_str(&data).expect("Got invalid message");
                    rx_callback(msg).await;
                }
            })
        });

        Self {
            tx,
            // rx,
            // ws_stream,
            // write,
            // read.
        }
    }

    pub async fn send(&mut self, message: SerializableMessage) {
        self.tx
            .clone()
            .unbounded_send(Message::binary(
                serde_json::to_string(&message)
                    .expect("Failed to convert SerializableMessage to str"),
            ))
            .expect("Failed to send message");
    }
}
