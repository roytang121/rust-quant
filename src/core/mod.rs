pub mod config;

#[async_trait]
pub trait OrderGateway {
    fn new() -> Self;
    async fn subscribe(&self) -> Result<(), Box<dyn std::error::Error>>;
}
