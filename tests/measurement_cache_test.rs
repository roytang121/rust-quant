#[cfg(test)]


#[cfg(test)]
mod test_common;

#[cfg(test)]
mod measurement_cache_test {
    use super::*;
    use rust_quant::model::constants::Exchanges;
    use rust_quant::model::Measurement::MMBasis;
    use rust_quant::model::{MeasurementCache, OrderSide, TSOptions};
    use std::sync::Arc;
    use test_common::common::*;

    #[tokio::test]
    async fn can_init() {
        before_each();
        let measurement_cache = Arc::new(MeasurementCache::new().await);
        let measurement = Arc::new(MMBasis {
            options: TSOptions::default(),
            exchange: Exchanges::Unknown,
            market: "ETH-PERP".to_string(),
            side: OrderSide::Buy,
        });
        measurement_cache.measurement(&measurement).await;
        for _i in 0..10000 {
            let measurement_cache = measurement_cache.clone();
            let measurement = measurement.clone();
            let time_ms = chrono::Utc::now().timestamp_millis();
            tokio::spawn(async move {
                measurement_cache
                    .add_point(&measurement, time_ms, 1.1)
                    .await;
            });
        }
        sleep(2000).await;
    }
}
