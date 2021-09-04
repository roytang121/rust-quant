#[cfg(test)]
mod rest_tests {
    use crate::ftx::FtxRestClient;

    use crate::ftx::types::FtxPlaceOrder;
    use crate::model::constants::Exchanges;
    use crate::model::{OrderRequest, OrderSide, OrderType};

    #[tokio::test]
    async fn it_init() {
        env_logger::init();
        std::env::set_var("ENV", "development");
        std::env::set_var("RUST_LOG", "INFO,DEBUG");
        let client = FtxRestClient::new();
        let response = client.get_account().await.unwrap();
        println!("{:?}", response);
        assert_eq!(2, 2)
    }

    #[tokio::test]
    async fn it_csat() {
        let order_request = OrderRequest {
            exchange: Exchanges::FTX,
            market: "ETH-PERP".to_string(),
            side: OrderSide::Buy,
            price: 123.123,
            size: 234.234,
            type_: OrderType::Limit,
            ioc: false,
            post_only: false,
            client_id: None,
        };
        let ftx_request = FtxPlaceOrder::from_order_request(order_request);
        println!("{:?}", ftx_request);
        println!("{:?}", serde_json::to_value(ftx_request).unwrap());
    }
}
