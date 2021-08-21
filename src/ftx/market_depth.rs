use std::collections::HashSet;

use futures_util::stream::SplitStream;
use futures_util::StreamExt;
use serde_json::{json, Value};
use simd_json::Error;
use tokio::net::TcpStream;
use tokio_stream::wrappers::ReceiverStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

use crate::ftx::types::{OrderBookData, WebSocketResponse};
use crate::ftx::utils::{connect_ftx, ping_pong};
use crate::model::market_data_model::{OrderBookSnapshot, PriceLevel};
use crate::pubsub::simple_message_bus::RedisBackedMessageBus;
use crate::pubsub::MessageBus;
use serde::Serialize;

pub fn process_orderbook_update(
    update: &OrderBookData,
    bid_ob: &mut HashSet<PriceLevel>,
    ask_ob: &mut HashSet<PriceLevel>,
) {
    for bid in update.bids.iter() {
        let price = bid[0];
        let size = bid[1];
        let price_level = PriceLevel {
            price: price,
            size: size,
        };
        if size > 0.0 {
            bid_ob.insert(price_level);
        } else {
            bid_ob.remove(&price_level);
        }
    }
    for ask in update.asks.iter() {
        let price = ask[0];
        let size = ask[1];
        let price_level = PriceLevel {
            price: price,
            size: size,
        };
        if size > 0.0 {
            ask_ob.insert(price_level);
        } else {
            ask_ob.remove(&price_level);
        }
    }
}

pub fn keys_of_hashset(hs: &HashSet<PriceLevel>, desc: bool) -> Vec<PriceLevel> {
    let mut keys: Vec<PriceLevel> = Vec::new();
    for key in hs.iter() {
        if key.size > 0.0 {
            keys.push(Clone::clone(key))
        }
    }
    keys.sort_by(|left, right| left.price.partial_cmp(&right.price).unwrap());
    if desc {
        keys.reverse();
    }
    keys
}

pub async fn subscribe_message(
    stream: &mut SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut bids_ob = HashSet::<PriceLevel>::new();
    let mut asks_ob = HashSet::<PriceLevel>::new();
    let mut message_bus = RedisBackedMessageBus::new().await?;
    while let Some(msg) = stream.next().await {
        let msg = msg?;
        let time_start_ms = chrono::Utc::now().timestamp_millis();
        let parse_result =
            simd_json::from_slice::<WebSocketResponse<OrderBookData>>(&mut *msg.into_data());
        match parse_result {
            Ok(response) => {
                log::debug!("{:?}", response);
                if let Some(orderbook_data) = response.data {
                    process_orderbook_update(&orderbook_data, &mut bids_ob, &mut asks_ob);
                    let snapshot = OrderBookSnapshot {
                        timestamp: chrono::Utc::now().timestamp_millis(),
                        bids: keys_of_hashset(&bids_ob, true),
                        asks: keys_of_hashset(&asks_ob, false),
                    };
                    let market = response.market.expect("missing market");
                    log::debug!("{:?}", snapshot);

                    let time_end_ms = chrono::Utc::now().timestamp_millis();
                    log::debug!(
                        "process marketdepth before sent: {} ms",
                        time_end_ms - time_start_ms
                    );

                    let result = message_bus
                        .publish(format!("marketdepth:ftx:{}", market).as_str(), &snapshot)
                        .await?;
                } else {
                    log::info!("{:?}", response);
                }
            }
            Err(err) => {
                log::error!("Error parsing OrderBookData. Error: {}", err);
            }
        }
    }
    Ok(())
}

pub async fn market_depth(market: &str) -> Result<(), Box<dyn std::error::Error>> {
    let (write, mut sub) = connect_ftx().await?;
    let (msg_tx, mut rx) = tokio::sync::mpsc::channel(32);
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
        "channel": "orderbook",
        "market": market,
    });

    msg_tx.send(Message::Text(init_message.to_string())).await;

    tokio::select! {
        Err(err) = subscribe_message(&mut sub) => {
            log::error!("subscribe_message error: {}", err);
        },
        _ = forward_write_to_ws => {
        }
        _ = ping_pong(msg_tx) => {},
    }
    Ok(())
}
