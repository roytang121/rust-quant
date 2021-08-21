use chrono::prelude::*;
use postgres::NoTls;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

use simple_error::bail;
use std::collections::HashMap;
use std::error::Error;
use std::time::SystemTime;

#[derive(Serialize, Deserialize, Debug)]
struct ApiResponse<T> {
    success: bool,
    #[serde(default)]
    result: T,
    #[serde(default)]
    error: String,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
struct TradeHistory {
    id: i64,
    price: f32,
    side: String,
    size: f32,
    time: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let market = "FTT/USD";
    let exchange = "FTX";
    let url = format!(
        "https://ftx.com/api/markets/{market}/trades",
        market = market
    );
    let client = reqwest::blocking::Client::new();
    let mut db = postgres::Client::connect("postgresql://admin:admin@localhost/admin_db", NoTls)?;
    let mut params: HashMap<String, i64> = HashMap::new();

    // let start_time = Utc.ymd(2020, 08, 11);
    // let start_time = NaiveDate::from_ymd(2020, 8, 11).and_hms(0, 0, 0);
    let end_time = Utc::now();
    // params.insert(String::from("start_time"), (start_time.timestamp()));
    params.insert(String::from("end_time"), end_time.timestamp());

    let all_trades: Vec<TradeHistory> = vec![];
    loop {
        // println!("{}", params["start_time"]);
        println!("{}", params["end_time"]);
        let resp = get_trades(&client, &url, &params)?;
        // println!("{:#?}", resp);
        if !resp.is_empty() {
            let last_end_time = resp
                .iter()
                .map(|trade| {
                    DateTime::parse_from_rfc3339(trade.time.as_str())
                        .expect("cannot parse datetime")
                })
                .min()
                .expect("error computing max");
            params.insert(String::from("end_time"), last_end_time.timestamp());
            for trade in resp {
                let _cmd = db.execute(
                    "insert into trades (id, exchange, market, price, side, size, time) values ($1, $2, $3, $4, $5, $6, $7)",
                    &[&trade.id, &exchange, &market, &trade.price, &trade.side, &trade.size, &SystemTime::from(DateTime::parse_from_rfc3339(trade.time.as_str())?)],
                ).ok();
                // all_trades.push(trade);
            }
            // println!("{}", all_trades.len());
        } else {
            break ();
        }
    }

    for x in all_trades {
        println!("{}", x.size)
    }
    Ok(())
}

fn get_trades(
    client: &Client,
    url: &str,
    params: &HashMap<String, i64>,
) -> Result<Vec<TradeHistory>, Box<dyn Error>> {
    let req = client.get(url).query(&params);
    let resp = req.send()?;
    let json = resp.json::<ApiResponse<Vec<TradeHistory>>>()?;
    match &json.success {
        true => Ok(json.result),
        false => {
            bail!(json.error)
        }
    }
}
