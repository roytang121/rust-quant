use serde::{Deserialize, Serialize};
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    pub redis_url: String,
    pub redis_ts_url: String,
    pub(crate) ftx_api_key: String,
    pub(crate) ftx_api_secret: String,
    pub(crate) ftx_sub_account: String,
}

pub struct ConfigStore {
    cfg: Config,
}

impl ConfigStore {
    pub fn new() -> ConfigStore {
        let environment = std::env::var("ENV").expect("ENV is not defined");
        let cfg: Config = confy::load_path(format!(
            "./environments/{}.config.toml",
            environment.to_lowercase()
        ))
        .unwrap();
        // log::info!("Loaded config: {:#?}", cfg);
        ConfigStore { cfg }
    }
    pub fn load() -> Config {
        ConfigStore::new().cfg
    }
}
