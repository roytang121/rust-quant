#[cfg(test)]
mod rest_tests {
    use crate::ftx::FtxRestClient;
    use std::collections::hash_map::RandomState;
    use std::collections::HashMap;
    use serde_json::Value;
    use std::error::Error;

    #[tokio::test]
    async fn it_init() {
        env_logger::init();
        std::env::set_var("ENV", "development");
        std::env::set_var("RUST_LOG", "INFO,DEBUG");
        std::env::set_var("RUST_BACKTRACE", "full");
        let client = FtxRestClient::new();
        let response = client.get_account().await.unwrap();
        println!("{:?}", response);
        assert_eq!(2, 2)
    }
}
