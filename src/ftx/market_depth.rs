use futures_util::stream::SplitStream;
use futures_util::StreamExt;
use serde_json::json;
use std::collections::{BTreeMap, HashSet};

use tokio::net::TcpStream;
use tokio_stream::wrappers::ReceiverStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

use crate::ftx::types::{FtxOrderBookData, WebSocketResponse};
use crate::ftx::utils::{connect_ftx, format_float, ping_pong};
use crate::model::constants::{Exchanges, PublishChannel};
use crate::model::market_data_model::{MarketDepth, PriceLevel};
use crate::pubsub::simple_message_bus::RedisBackedMessageBus;
use crate::pubsub::PublishPayload;
use ordered_float::OrderedFloat;
use std::cmp::{max, min};

type PriceMap = BTreeMap<OrderedFloat<f64>, f64>;

pub fn process_orderbook_update(
    update: &FtxOrderBookData,
    bid_ob: &mut PriceMap,
    ask_ob: &mut PriceMap,
) {
    for bid in update.bids.iter() {
        let price = bid[0];
        let size = bid[1];
        if size > 0.0 {
            bid_ob.insert(price.into(), size);
        } else {
            bid_ob.remove(&price.into());
        }
    }
    for ask in update.asks.iter() {
        let price = ask[0];
        let size = ask[1];
        if size > 0.0 {
            ask_ob.insert(price.into(), size);
        } else {
            ask_ob.remove(&price.into());
        }
    }
}

#[deprecated]
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

pub fn keys_of_btree(btree: &PriceMap, desc: bool) -> Vec<PriceLevel> {
    let mut clone: Vec<PriceLevel> = btree
        .iter()
        .map(|price| PriceLevel {
            price: price.0.into_inner(),
            size: price.1.to_owned(),
        })
        .collect();
    if desc {
        clone.reverse()
    }
    clone
}

pub fn validate_checksum(checksum: u32, md: &MarketDepth) -> bool {
    let bids = &md.bids;
    let asks = &md.asks;
    let max_len = min(100, max(bids.len(), asks.len()));
    let mut arr: Vec<String> = Vec::new();
    for i in 0..max_len {
        if let Some(bid) = bids.get(i) {
            arr.push(format_float(&bid.price));
            arr.push(format_float(&bid.size));
        }
        if let Some(ask) = asks.get(i) {
            arr.push(format_float(&ask.price));
            arr.push(format_float(&ask.size));
        }
    }
    let sign: String = arr.join(":");
    let mut hasher = crc32fast::Hasher::new();
    hasher.update(sign.as_bytes());
    let crc = hasher.finalize();
    log::debug!("checksum: {}, crc: {}, sign: {}", checksum, crc, sign);
    return checksum == crc;
}

pub async fn subscribe_message(
    stream: &mut SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    message_bus_sender: &tokio::sync::mpsc::Sender<PublishPayload>,
) -> anyhow::Result<()> {
    let mut bids_ob = PriceMap::new();
    let mut asks_ob = PriceMap::new();
    while let Some(msg) = stream.next().await {
        let msg = msg?;
        let time_start_ns = chrono::Utc::now().timestamp_nanos();
        let parse_result =
            serde_json::from_slice::<WebSocketResponse<FtxOrderBookData>>(&mut *msg.into_data());
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
                        bids: keys_of_btree(&bids_ob, true),
                        asks: keys_of_btree(&asks_ob, false),
                    };
                    if !validate_checksum(orderbook_data.checksum, &snapshot) {
                        return Err(anyhow!("Checksum failed. aborting..."));
                    }
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

                    if let Err(err) = message_bus_sender.send(payload).await {
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
    let (msg_tx, rx) = tokio::sync::mpsc::channel(32);
    let forward_write_to_ws = ReceiverStream::new(rx)
        .map(|x| {
            log::info!("send {}", x);
            return x;
        })
        .map(Ok)
        .forward(write);

    // message bus instance
    let message_bus = RedisBackedMessageBus::new().await?;
    let message_bus_sender = message_bus.publish_tx.clone();
    // init message
    let init_message = json!({
        "op": "subscribe",
        "channel": "orderbook",
        "market": market,
    });
    msg_tx.send(Message::Text(init_message.to_string())).await;

    // polling message bus publisher
    let message_bus_poll = message_bus.subscribe();

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
