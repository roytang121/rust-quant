use rust_quant::view::view_service::{LambdaViewService};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let service = LambdaViewService::new();
    return match service.subscribe().await {
        Ok(_) => {
            log::error!("view exited uncaught");
            Ok(())
        }
        Err(err) => {
            log::error!("{}", err);
            Err(anyhow::anyhow!(err))
        }
    };
}
