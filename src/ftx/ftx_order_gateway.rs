use crate::core::OrderGateway;
use crate::ftx::types::{FtxOrderData, WebSocketResponse, WebSocketResponseType};
use crate::ftx::utils::{connect_ftx_authed, ping_pong};
use crate::ftx::FtxRestClient;
use crate::model::constants::{Exchanges, PublishChannel};
use crate::model::{CancelOrderRequest, OrderRequest};
use crate::pubsub::simple_message_bus::{MessageConsumer, RedisBackedMessageBus};
use async_trait::async_trait;
use dashmap::DashMap;
use futures_util::stream::SplitStream;
use futures_util::{SinkExt, StreamExt};
use redis::Commands;
use serde_json::json;

use std::error::Error;
use std::sync::Arc;

use thiserror::Error;
use tokio::net::TcpStream;

use tokio::sync::RwLock;
use tokio_stream::wrappers::ReceiverStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

pub struct FtxOrderGateway {
    order_cache: Arc<DashMap<String, String>>,
}

#[derive(Error, Debug)]
enum OrderGatewayError {
    #[error("unknown data store error")]
    Unknown,
}

impl FtxOrderGateway {
    pub async fn process_message(
        &self,
        stream: &mut SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut redis = RedisBackedMessageBus::new().await?;
        while let Some(msg) = stream.next().await {
            let msg = msg?;
            let response = serde_json::from_str::<WebSocketResponse<FtxOrderData>>(
                msg.to_string().as_mut_str(),
            )?;
            match response.channel {
                None => {
                    continue;
                }
                Some(ref channel) => {
                    if channel != "orders" {
                        continue;
                    }
                }
            }
            match response.type_ {
                WebSocketResponseType::error => {
                    log::error!("error {:?}", response);
                    return Err(Box::new(OrderGatewayError::Unknown));
                }
                WebSocketResponseType::update => {
                    log::info!("{:?}", response);
                    if let Some(data) = response.data {
                        let order_update = data.to_order_update();
                        redis
                            .publish(
                                PublishChannel::OrderUpdate.to_string().as_str(),
                                &order_update,
                            )
                            .await;
                    }
                }
                _ => {
                    log::info!("{:?}", response);
                }
            }
        }
        Ok(())
    }
}

#[async_trait]
impl OrderGateway for FtxOrderGateway {
    fn new() -> FtxOrderGateway {
        FtxOrderGateway {
            order_cache: Arc::new(DashMap::new()),
        }
    }

    async fn subscribe(&self) -> Result<(), Box<dyn std::error::Error>> {
        let (mut write, mut read) = connect_ftx_authed().await?;
        let (tx, rx) = tokio::sync::mpsc::channel(32);
        let init_message = json!({
            "op": "subscribe",
            "channel": "orders",
        });
        log::info!("{}", init_message);
        write.send(Message::Text(init_message.to_string())).await;
        let forward_write_to_ws = ReceiverStream::new(rx).map(Ok).forward(write);

        let order_request_service = FtxOrderRequestService::new();
        let cancel_order_service = FtxCancelOrderService::new();
        tokio::select! {
            Err(err) = self.process_message(&mut read) => {
                log::error!("ftx_order_gateway process_message panic: {}", err)
            },
            Err(err) = order_request_service.subscribe() => {
                log::error!("order_request_service panic: {}", err)
            },
            Err(err) = cancel_order_service.subscribe() => {
                log::error!("cancel_order_service panic: {}", err)
            },
            _ = forward_write_to_ws => {},
            _ = ping_pong(tx) => {},
        }
        Ok(())
    }
}

struct FtxOrderRequestService {
    client: Arc<RwLock<FtxRestClient>>,
}
impl FtxOrderRequestService {
    pub fn new() -> Self {
        let client = Arc::new(RwLock::new(FtxRestClient::new()));
        FtxOrderRequestService { client }
    }
    pub async fn subscribe(&self) -> Result<(), Box<dyn std::error::Error>> {
        RedisBackedMessageBus::subscribe_channels(
            vec![PublishChannel::OrderRequest.as_ref()],
            self,
        )
        .await?;
        Ok(())
    }
    async fn accept_order_request(&self, order_request: OrderRequest) {
        let client_ref = self.client.clone();
        // parallel
        tokio::spawn(async move {
            let client = client_ref.read().await;
            match client.place_order(order_request).await {
                Ok(_response) => {}
                Err(_err) => {}
            }
        });
    }
}
#[async_trait]
impl MessageConsumer for FtxOrderRequestService {
    async fn consume(&self, msg: &mut str) -> anyhow::Result<()> {
        match serde_json::from_str::<OrderRequest>(msg) {
            Ok(order_request) => {
                if order_request.exchange == Exchanges::FTX {
                    log::info!("FtxOrderRequestService: {:?}", order_request);
                    self.accept_order_request(order_request).await;
                }
            }
            Err(err) => {
                log::error!("{}", err)
            }
        };
        Ok(())
    }
}

struct FtxCancelOrderService {
    client: Arc<RwLock<FtxRestClient>>,
}
impl FtxCancelOrderService {
    pub fn new() -> Self {
        let client = Arc::new(RwLock::new(FtxRestClient::new()));
        FtxCancelOrderService { client }
    }
    pub async fn subscribe(&self) -> Result<(), Box<dyn std::error::Error>> {
        RedisBackedMessageBus::subscribe_channels(vec![PublishChannel::CancelOrder.as_ref()], self)
            .await?;
        Ok(())
    }
    async fn accept_cancel_order_request(&self, cancel_order_request: CancelOrderRequest) {
        let client_ref = self.client.clone();
        // parallel
        tokio::spawn(async move {
            let client = client_ref.read().await;
            match client
                .cancel_order_cid(cancel_order_request.client_id.as_str())
                .await
            {
                Ok(_response) => {}
                Err(_err) => {}
            }
        });
    }
}
#[async_trait]
impl MessageConsumer for FtxCancelOrderService {
    async fn consume(&self, msg: &mut str) -> anyhow::Result<()> {
        match serde_json::from_str::<CancelOrderRequest>(msg) {
            Ok(order_request) => {
                if order_request.exchange == Exchanges::FTX {
                    log::info!("FtxCancelOrderService: {:?}", order_request);
                    self.accept_cancel_order_request(order_request).await;
                }
            }
            Err(err) => {
                log::error!("{}", err)
            }
        };
        Ok(())
    }
}
