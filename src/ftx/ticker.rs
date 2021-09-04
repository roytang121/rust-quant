use std::error::Error;

use futures_util::stream::SplitStream;
use futures_util::{AsyncWrite, SinkExt, StreamExt, TryFutureExt};
use redis::aio::Connection;
use serde_json::{json, Value};
use tokio::net::TcpStream;

use tokio_stream::wrappers::ReceiverStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use tungstenite::Message;

use crate::ftx::utils::{connect_ftx, ping_pong};

pub async fn subscribe_message(
    stream: &mut SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
) -> Result<(), Box<dyn Error>> {
    let redis_client = redis::Client::open("redis://localhost:10400")?;
    let mut conn = redis_client.get_async_connection().await?;
    while let Some(msg) = stream.next().await {
        let msg = msg?;
        let msg_json = serde_json::from_str::<Value>(msg.to_string().as_mut_str())?;
        log::debug!("{}", msg.to_string());
        if msg_json["data"].is_object() && msg_json["data"]["time"].is_f64() {
            if let Some(time_ms) = msg_json["data"]["time"].as_f64() {
                let market = msg_json["market"].as_str().unwrap();
                let now = chrono::Utc::now();
                let _time_diff = now.timestamp_millis() - (time_ms * 1000f64).round() as i64;
                // log::info!("time_diff: {}", now.timestamp_millis() - (time_ms * 1000f64).round() as i64);
                // conn.publish(format!("ticker:ftx:{}", market), msg.to_string());
                redis::cmd("PUBLISH")
                    .arg(format!("ticker:ftx:{}", market))
                    .arg(msg.to_string().as_str())
                    .query_async::<Connection, String>(&mut conn)
                    .await;
            }
        }
    }
    Ok(())
}

pub async fn ticker(market: &str) -> Result<(), Box<dyn Error>> {
    let (write, mut sub) = connect_ftx().await?;
    let (msg_tx, rx) = tokio::sync::mpsc::channel(32);
    let forward_write_to_ws = ReceiverStream::new(rx)
        .map(|x| {
            log::info!("send {}", x);
            return x;
        })
        .map(Ok)
        .forward(write);
    // init message
    let init_message = json!({
        "op": "subscribe",
        "channel": "ticker",
        "market": &market,
    });

    msg_tx.send(Message::Text(init_message.to_string())).await;

    tokio::select! {
        Err(err) = subscribe_message(&mut sub) => {
            eprintln!("{}", err);
        },
        _ = forward_write_to_ws => {
        }
        _ = ping_pong(msg_tx) => {},
    }
    Ok(())
}
