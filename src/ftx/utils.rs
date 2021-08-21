use std::error::Error;
use std::time::Duration;

use futures_util::stream::{SplitSink, SplitStream};
use futures_util::StreamExt;
use serde_json::json;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

use crate::conn::websocket::connect_wss_async;

pub(crate) async fn ping_pong(
    write: tokio::sync::mpsc::Sender<Message>,
) -> Result<(), Box<dyn Error>> {
    loop {
        let ping = json!({"op": "ping"});
        write.send(Message::Text(ping.to_string())).await;
        tokio::time::sleep(Duration::from_secs(10)).await;
    }
}

pub async fn connect_ftx() -> Result<
    (
        SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
        SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    ),
    Box<dyn Error>,
> {
    let url = "wss://ftx.com/ws/";
    let socket = connect_wss_async(&url).await?;
    let (write, read) = socket.split();
    // auth
    // init message
    Ok((write, read))
}
