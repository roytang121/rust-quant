use tonic::{transport::Server, Request, Response, Status};

use lambda_view::lambda_view_server::{LambdaView, LambdaViewServer};
use lambda_view::{HelloReply, HelloRequest, ParamsRequest, ParamsResponse};
use std::time::Duration;
use tokio_stream::wrappers::ReceiverStream;
use crate::pubsub::simple_message_bus::{RedisBackedMessageBus, MessageConsumer};
use std::sync::Arc;

pub mod lambda_view {
    tonic::include_proto!("lambda_view");
}

#[derive(Debug, Default)]
pub struct LambdaViewImpl {}

#[tonic::async_trait]
impl LambdaView for LambdaViewImpl {
    type StreamParamsStream = ReceiverStream<Result<ParamsResponse, Status>>;

    async fn say_hello(
        &self,
        request: Request<HelloRequest>, // Accept request of type HelloRequest
    ) -> Result<Response<HelloReply>, Status> {
        // Return an instance of type HelloReply
        info!("Got a request: {:?}", request);

        let reply = HelloReply {
            message: format!("Hello {}!", request.into_inner().name).into(), // We must use .into_inner() as the fields of gRPC requests and responses are private
        };

        Ok(Response::new(reply)) // Send back our formatted greeting
    }

    async fn stream_params(
        &self,
        request: Request<ParamsRequest>,
    ) -> Result<Response<Self::StreamParamsStream>, Status> {
        info!("Got a request: {:?}", request);
        let (mut tx, rx) = tokio::sync::mpsc::channel(4);
        let subscribe_key = request.into_inner().key;
        tokio::spawn(async move {
            let consumer = LambdaParamsMessageConsumer(tx);
            RedisBackedMessageBus::subscribe_channels(vec![&subscribe_key], &consumer).await.unwrap();
        });
        Ok(Response::new(ReceiverStream::new(rx)))
    }
}

struct LambdaParamsMessageConsumer(tokio::sync::mpsc::Sender<Result<ParamsResponse, Status>>);
#[async_trait::async_trait]
impl MessageConsumer for LambdaParamsMessageConsumer {
    async fn consume(&self, msg: &[u8]) -> anyhow::Result<()> {
        let message = std::str::from_utf8(msg).unwrap();
        let response = ParamsResponse {
            json: message.to_string()
        };
        return match self.0.send(Ok(response)).await {
            Ok(_) => {
                Ok(())
            },
            Err(err) => {
                error!("{}", err);
                Err(anyhow::anyhow!(err))
            }
        }
    }
}

pub struct LambdaViewService {}
impl LambdaViewService {
    pub fn new() -> Self {
        LambdaViewService {}
    }
    pub async fn subscribe(&self) -> anyhow::Result<()> {
        let socket_addr = "0.0.0.0:50051".parse()?;
        let lambda_view_grpc = LambdaViewImpl::default();
        Server::builder()
            .accept_http1(true)
            .add_service(tonic_web::enable(LambdaViewServer::new(lambda_view_grpc)))
            .serve(socket_addr)
            .await?;

        Ok(())
    }
}
