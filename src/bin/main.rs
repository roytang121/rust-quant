use nng::*;

use rmp_serde::Serializer;
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::HashSet;

use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tungstenite::{connect, Message};

mod types;
use std::sync::{Arc, Mutex};
use types::*;

fn get_redis() -> redis::Connection {
    let redis_client =
        redis::Client::open("redis://localhost:10400").expect("Failed connecting to Redis client");
    let con = redis_client
        .get_connection()
        .expect("Failed getting connection from redis");
    return con;
}

struct Counter {
    value: i32,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let args: Vec<String> = std::env::args().collect();
    let market = args[1].to_owned();
    // redis client
    let _redis_client = get_redis();
    // nng client
    let nng_client = Socket::new(Protocol::Pub0).expect("failed to open nng socket");
    nng_client
        .listen("tcp://127.0.0.1:10404")
        .expect("failed to listen nng inproc");

    let ws_host = "wss://ftx.com/ws/";
    let (mut socket, response) =
        connect(url::Url::parse(ws_host).unwrap()).expect("cannot connect");
    println!("Connected to conn");
    println!("Response HTTP code {}", response.status());
    for (header, _value) in response.headers() {
        println!("* {}", header);
    }

    let init_message = json!({
        "op": "subscribe",
        "channel": "orderbook",
        "market": market.as_str(),
    })
    .to_string();

    socket.write_message(Message::Text(init_message)).unwrap();

    // orderbook hashmap
    let mut bid_ob: HashSet<PriceLevel> = HashSet::new();
    let mut ask_ob: HashSet<PriceLevel> = HashSet::new();
    let mut orderbook_snapshot: OrderBookSnapshot = OrderBookSnapshot {
        timestamp: get_time(),
        latency: 0.0,
        bids: keys_of_hashset(&bid_ob, true),
        asks: keys_of_hashset(&ask_ob, false),
    };

    let (tx, mut rx) = tokio::sync::mpsc::channel(32);

    let counter = Arc::new(Mutex::new(Counter { value: 0 }));
    let counter_ref = counter.clone();
    tokio::spawn(async move {
        while let Some(_v) = rx.recv().await {
            let mut counter = counter.lock().unwrap();
            counter.value += 1;
        }
    });
    loop {
        tx.send(1).await.unwrap();
        let msg = socket.read_message().expect("Error reading message");
        let orderbook_message: Value =
            serde_json::from_str(msg.to_string().as_str()).expect("Cannot parse orderbook update");
        if orderbook_message["type"] == "update" && orderbook_message["channel"] == "orderbook" {
            let orderbook_update: OrderbookUpdate =
                serde_json::from_value(orderbook_message["data"].clone())
                    .expect("Cannot parse orderbook data");
            // println!("Received: {:.30}", orderbook_update.time);
            process_orderbook_update(&orderbook_update, &mut bid_ob, &mut ask_ob);
            orderbook_snapshot.timestamp = get_time();
            orderbook_snapshot.latency = (get_time() as f64) - (orderbook_update.time * 1000.0);
            orderbook_snapshot.bids = keys_of_hashset(&bid_ob, true);
            orderbook_snapshot.asks = keys_of_hashset(&ask_ob, false);
            // publish_redis_channel(&mut redis_client, "ftx:eth-perp", &orderbook_snapshot).unwrap();
            publish_nng(&nng_client, &orderbook_snapshot);
            stat_orderbook_snapshot(&orderbook_snapshot, 3.1);
            // println!("bids size: {}", orderbook_snapshot.bids.len());
        }
    }

    Ok(())
}

pub fn publish_redis_channel(
    con: &mut redis::Connection,
    channel: &str,
    message: &OrderBookSnapshot,
) -> redis::RedisResult<()> {
    let result = redis::cmd("publish")
        .arg(channel)
        .arg(pack_json(message))
        .query(con);
    return result;
}

pub fn publish_nng(nng: &Socket, message: &OrderBookSnapshot) {
    nng.send(pack_json(message).as_bytes())
        .expect("failed to send nng");
}

pub fn pack_json<T: Serialize>(value: &T) -> String {
    serde_json::to_string(&value).expect("Failed to serialize value")
}

pub fn pack_value<T: Serialize>(value: &T) -> Vec<u8> {
    let mut buf = Vec::new();
    value
        .serialize(&mut Serializer::new(&mut buf))
        .expect("Failed to serialize value");
    buf
}

pub fn process_orderbook_update(
    update: &OrderbookUpdate,
    bid_ob: &mut HashSet<PriceLevel>,
    ask_ob: &mut HashSet<PriceLevel>,
) {
    for bid in update.bids.iter() {
        let price = bid[0];
        let size = bid[1];
        let price_level = PriceLevel {
            price_f: price,
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
            price_f: price,
            size: size,
        };
        if size > 0.0 {
            ask_ob.insert(price_level);
        } else {
            ask_ob.remove(&price_level);
        }
    }
}

pub fn stat_orderbook_snapshot(ob: &OrderBookSnapshot, target_size: f32) {
    if ob.asks.is_empty() || ob.bids.is_empty() {
        return;
    }
    let mut sum_ask: f32 = 0.0;
    let mut price: f32 = -1.0;
    let mut level: i32 = 0;
    for ask in ob.asks.iter() {
        sum_ask += ask.size;
        level += 1;
        if sum_ask >= target_size {
            price = ask.price_f;
            break;
        }
    }
    let cur_price = ob.asks[0].price_f;
    let abs_diff = price - cur_price;
    let bps = (abs_diff / cur_price) * 10000.0;
    println!("current: {}, target: {}", cur_price, price);
    println!("bps: {}, level: {}", bps, level);
}

pub fn keys_of_hashset(hs: &HashSet<PriceLevel>, desc: bool) -> Vec<PriceLevel> {
    let mut keys: Vec<PriceLevel> = Vec::new();
    for key in hs.iter() {
        if key.size > 0.0 {
            keys.push(Clone::clone(key))
        }
    }
    keys.sort_by(|left, right| left.price_f.partial_cmp(&right.price_f).unwrap());
    if desc {
        keys.reverse();
    }
    keys
}

pub fn get_time() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis()
}
