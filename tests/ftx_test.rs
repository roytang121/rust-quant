#[cfg(test)]
mod test_common;

#[cfg(test)]
mod ftx_test {
    use rust_quant::model::{OrderRequest, OrderSide, OrderType};
    use rust_quant::model::constants::Exchanges;
    use rust_quant::ftx::{FtxPlaceOrder, FtxOrderStatus, FtxOrderSide, FtxOrderType};

    #[test]
    pub fn transform_place_market_order() {
        let order_request = OrderRequest {
            exchange: Exchanges::FTX,
            market: "unknown".to_string(),
            side: OrderSide::Buy,
            price: 10.0,
            size: 1.0,
            type_: OrderType::Market,
            ioc: true,
            post_only: true,
            client_id: None
        };
        let fo = FtxPlaceOrder::from(order_request);
        assert!(matches!(fo.type_, FtxOrderType::market));
        assert_eq!(fo.postOnly, false);
        assert_eq!(fo.ioc, false);
        assert_eq!(fo.price, None);
        assert!(matches!(fo.side, FtxOrderSide::buy))
    }

    #[test]
    pub fn transform_place_limit_order() {
        let order_request = OrderRequest {
            exchange: Exchanges::FTX,
            market: "unknown".to_string(),
            side: OrderSide::Sell,
            price: 10.0,
            size: 1.0,
            type_: OrderType::Limit,
            ioc: true,
            post_only: true,
            client_id: None
        };
        let fo = FtxPlaceOrder::from(order_request);
        assert!(matches!(fo.type_, FtxOrderType::limit));
        assert_eq!(fo.postOnly, true);
        assert_eq!(fo.ioc, true);
        assert_eq!(fo.price, None);
        assert!(matches!(fo.side, FtxOrderSide::buy))
    }
}