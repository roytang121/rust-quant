use futures_util::stream::SplitStream;
use futures_util::StreamExt;
use serde_json::{json, Value};
use simd_json::Error;
use std::collections::HashSet;
use strum_macros;
use tokio::net::TcpStream;
use tokio_stream::wrappers::ReceiverStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

use crate::ftx::types::{OrderBookData, WebSocketResponse};
use crate::ftx::utils::{connect_ftx, ping_pong};
use crate::model::constants::{Exchanges, PublishChannel};
use crate::model::market_data_model::{MarketDepth, PriceLevel};
use crate::pubsub::simple_message_bus::RedisBackedMessageBus;
use crate::pubsub::{MessageBus, MessageBusUtils, PublishPayload};
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
    message_bus_sender: &tokio::sync::mpsc::Sender<PublishPayload>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut bids_ob = HashSet::<PriceLevel>::new();
    let mut asks_ob = HashSet::<PriceLevel>::new();
    while let Some(msg) = stream.next().await {
        let msg = msg?;
        let time_start_ns = chrono::Utc::now().timestamp_nanos();
        let parse_result =
            simd_json::from_slice::<WebSocketResponse<OrderBookData>>(&mut *msg.into_data());
        match parse_result {
            Ok(response) => {
                log::debug!("{:?}", response);
                if let Some(orderbook_data) = response.data {
                    process_orderbook_update(&orderbook_data, &mut bids_ob, &mut asks_ob);
                    let market = response.market.expect("missing market");
                    let snapshot = MarketDepth {
                        exchange: Exchanges::FTX,
                        market: market.clone(),
                        timestamp: chrono::Utc::now().timestamp_millis(),
                        bids: keys_of_hashset(&bids_ob, true),
                        asks: keys_of_hashset(&asks_ob, false),
                    };
                    log::debug!("{:?}", snapshot);
                    let payload = PublishPayload {
                        channel: format!(
                            "{}:{}:{}",
                            PublishChannel::MarketDepth.to_string(),
                            Exchanges::FTX.to_string(),
                            market
                        ),
                        payload: serde_json::to_string(&snapshot)?,
                    };

                    if let Err(err) =
                        MessageBusUtils::publish_async(message_bus_sender, payload).await
                    {
                        log::error!("md process msg error: {}", err);
                    }

                    let time_end_ns = chrono::Utc::now().timestamp_nanos();
                    log::info!(
                        "process marketdepth after publish: {} nanos, {} ms",
                        time_end_ns - time_start_ns,
                        (time_end_ns - time_start_ns) as f32 * 0.000001
                    );
                    // let result = message_bus
                    // .publish(format!("marketdepth:{}:{}", Exchanges::FTX, market).as_str(), &snapshot)
                    // .publish_async(payload)
                    // .await?;
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

    // message bus instance
    let mut message_bus = RedisBackedMessageBus::new().await?;
    let message_bus_sender = message_bus.publish_tx.clone();
    // init message
    let init_message = json!({
        "op": "subscribe",
        "channel": "orderbook",
        "market": market,
    });
    msg_tx.send(Message::Text(init_message.to_string())).await;

    // polling message bus publisher
    let message_bus_poll = message_bus.publish_poll();

    tokio::select! {
        Err(err) = subscribe_message(&mut sub, &message_bus_sender) => {
            log::error!("subscribe_message error: {}", err);
        },
        Err(err) = forward_write_to_ws => {
            log::error!("forward_write_to_ws error: {}", err);
        }
        Err(err) = ping_pong(msg_tx) => {
            log::error!("ping_pong error: {}", err);
        },
        Err(err) = message_bus_poll => {
            log::error!("message_bus_poll error: {}", err);
        },
    }
    Ok(())
}
