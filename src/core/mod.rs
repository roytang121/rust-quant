pub mod config;

#[async_trait::async_trait]
pub trait OrderGateway {
    async fn subscribe(&self) -> anyhow::Result<()>;
}
