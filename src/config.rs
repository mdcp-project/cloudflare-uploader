use prelude::*;

#[derive(Clone, Debug, serde::Deserialize)]
pub struct Config {
    pub token: String,
    pub account_id: String,
}

impl Config {
    pub fn new() -> Result<Self> {
        log::info!("Parsing config");

        let mut cfg = config::Config::new();
        cfg.merge(config::Environment::with_prefix("UPLOADER"))?;

        cfg.try_into().map_err(|e| anyhow!(e))
    }
}
