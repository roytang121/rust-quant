use crate::core::config::ConfigStore;
use crate::model::constants::Exchanges;
use crate::model::OrderSide;
use redis::AsyncCommands;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TSOptions {
    retention: i64,
}
impl Default for TSOptions {
    fn default() -> Self {
        TSOptions { retention: 0 }
    }
}

#[derive(Serialize, Deserialize, Debug, strum_macros::Display, Clone)]
pub enum Measurement {
    MMBasis {
        options: TSOptions,
        exchange: Exchanges,
        market: String,
        side: OrderSide,
    },
}
trait FillRedisArgs {
    fn redis_args(&self) -> Vec<String>;
}
impl FillRedisArgs for TSOptions {
    fn redis_args(&self) -> Vec<String> {
        vec!["RETENTION".to_string(), self.retention.to_string()]
    }
}
impl FillRedisArgs for Measurement {
    fn redis_args(&self) -> Vec<String> {
        match self {
            Measurement::MMBasis {
                options,
                exchange,
                market,
                side,
            } => {
                let options_args = options.redis_args();
                let args = vec![
                    "LABELS".to_string(),
                    "exchange".to_string(),
                    exchange.to_string(),
                    "market".to_string(),
                    market.to_string(),
                    "side".to_string(),
                    side.to_string(),
                ];
                vec![options_args, args].concat()
            }
        }
    }
}

pub struct MeasurementCache {
    client: redis::Client,
    shared_conn: redis::aio::MultiplexedConnection,
}

impl MeasurementCache {
    pub async fn new() -> Self {
        let config = ConfigStore::load();
        let redis = redis::Client::open(config.redis_ts_url).expect("Failed to connect redis_ts");
        let conn = redis
            .get_multiplexed_async_connection()
            .await
            .expect("redis_ts: Failed to get multiplexed connection");
        MeasurementCache {
            client: redis,
            shared_conn: conn,
        }
    }

    pub async fn measurement(&self, measurement: &Measurement) -> &Self {
        let mut conn = self.shared_conn.clone();
        let measurement_name = measurement.to_string();
        match conn
            .keys::<&str, Vec<redis::Value>>(&measurement_name)
            .await
        {
            Ok(result) => match result.is_empty() {
                true => {
                    info!(
                        "measurement {:?} does not exists. creating...",
                        measurement_name
                    );
                    let measurement_args = measurement.redis_args();
                    let args = measurement_args
                        .iter()
                        .map(AsRef::as_ref)
                        .collect::<Vec<&str>>();
                    log::info!("{:?}", args);
                    redis::cmd("TS.CREATE")
                        .arg(measurement_name.as_str())
                        .arg(&args)
                        .query_async::<redis::aio::MultiplexedConnection, redis::Value>(&mut conn)
                        .await;
                }
                false => {
                    info!("measurement {:?} exists. altering...", measurement_name);
                    let measurement_args = measurement.redis_args();
                    let args = measurement_args
                        .iter()
                        .map(AsRef::as_ref)
                        .collect::<Vec<&str>>();
                    redis::cmd("TS.ALTER")
                        .arg(measurement_name.as_str())
                        .arg(&args)
                        .query_async::<redis::aio::MultiplexedConnection, redis::Value>(&mut conn)
                        .await;
                }
            },
            Err(err) => error!("{}", err),
        }
        self
    }

    #[deprecated]
    pub async fn add_point(&self, measurement: &Measurement, time_ms: i64, point: f64) {
        let mut conn = self.shared_conn.clone();
        let measurement = measurement.clone();
        let result = redis::cmd("ts.add")
            .arg(measurement.to_string())
            .arg(time_ms)
            .arg(point)
            .query_async::<redis::aio::MultiplexedConnection, redis::Value>(&mut conn)
            .await;
        match result {
            Ok(_) => {}
            Err(err) => error!("{}, key = {}", err, time_ms),
        }
    }

    pub fn add_point_now(&self, measurement: &'static Measurement, point: f64) {
        let mut conn = self.shared_conn.clone();
        let time_now = chrono::Utc::now().timestamp_millis();
        tokio::spawn(async move {
            let result = redis::cmd("ts.add")
                .arg(measurement.to_string())
                .arg(time_now)
                .arg(point)
                .query_async::<redis::aio::MultiplexedConnection, redis::Value>(&mut conn)
                .await;
            match result {
                Ok(_) => {}
                Err(err) => error!("{}, key = {}", err, time_now),
            }
        });
    }
}
