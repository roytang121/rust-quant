use std::time::Duration;

use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

use hmac::{Hmac, Mac, NewMac};
use sha2::Sha256;

use crate::conn::websocket::connect_wss_async;
use crate::core::config::ConfigStore;

pub(crate) async fn ping_pong(write: tokio::sync::mpsc::Sender<Message>) -> anyhow::Result<()> {
    loop {
        let ping = json!({"op": "ping"});
        write.send(Message::Text(ping.to_string())).await;
        tokio::time::sleep(Duration::from_secs(10)).await;
    }
}

pub async fn connect_ftx() -> anyhow::Result<(
    SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
)> {
    let url = "wss://ftx.com/ws/";
    let socket = connect_wss_async(&url).await?;
    let (write, read) = socket.split();
    Ok((write, read))
}

pub async fn connect_ftx_authed() -> anyhow::Result<(
    SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
)> {
    let cfg = ConfigStore::load();
    let (mut write, read) = connect_ftx().await?;
    let ts = chrono::Utc::now().timestamp_millis();
    let sign = generate_signature(cfg.ftx_api_secret.as_str(), ts);

    let auth_message = json!({
        "args": {
            "key": cfg.ftx_api_key,
            "sign": sign,
            "time": ts,
            "subaccount": cfg.ftx_sub_account,
        },
        "op": "login",
    });
    log::info!("auth: {}", auth_message);
    write.send(Message::Text(auth_message.to_string())).await?;
    Ok((write, read))
}

pub fn generate_signature(secret: &str, ts: i64) -> String {
    type HmacSha256 = Hmac<Sha256>;
    // TS implementation
    // const sign = CryptoJS.HmacSHA256(`${ts}websocket_login`, secret).toString();
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(format!("{}websocket_login", ts).as_bytes());

    let result = mac.finalize().into_bytes();

    return hex::encode(result);
}

/// 1   ->  1.0
/// 1.2 ->  1.2
pub fn format_float(val: &f64) -> String {
    match val.fract() > 0.0 {
        true => val.to_string(),
        false => val.to_string() + ".0",
    }
}
