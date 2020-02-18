
mod protocol;
mod kubernetes;
mod discovery;
mod proxy;
pub mod util;

use std::path::Path;
use std::fs;
use std::sync::Arc;

use tokio::task;
#[macro_use] extern crate anyhow;
use anyhow::Result;
#[macro_use] extern crate slog;
use slog::Logger;
use serde::{Deserialize};
use serde_json;
use schemars::JsonSchema;
use tokio::sync::RwLock;

use protocol::ServiceCoordinates;
use discovery::{
    DiscoveryOptions,
    run_endpoint_discovery
};
use proxy::{
    run_proxy
};

#[derive(Clone, Debug, Deserialize, JsonSchema)]
pub struct Config {
    pub port: u16,
    pub discovery: DiscoveryOptions,
}
impl Config {
    pub fn load_config(path: &Path) -> Result<Config> {
        let content = fs::read_to_string(path)?;
        serde_json::from_str(&content)
            .map_err(|e| anyhow!("{}", e))
    }
}

pub async fn do_run_singular(config: Config, log: Logger) -> Result<(), Box<dyn std::error::Error>> {
    info!(log, "starting singular...");

    // The RWLock used to communicate the URI to which forward requests to to the proxy
    let forward_uri_lock = Arc::new(RwLock::new(String::from("")));

    // Spawn task for endpoint discovery
    let log_c = log.clone();
    let forward_uri_lock_c = forward_uri_lock.clone();
    let config_c = config.clone();
    task::spawn(async move {
        let res = run_endpoint_discovery(config_c.discovery, log_c.clone(), forward_uri_lock_c).await;
        if let Err(e) = res {
            error!(log_c, "discovery error: {:?}", e);
        }
    });

    // Spawn task for running the actual proxy
    task::spawn(async move {
        println!("running server on {:?}", config.port);
        let res = run_proxy(config.port, forward_uri_lock).await;
        if let Err(e) = res {
            error!(log, "proxy error: {:?}", e);
        }
    });

    Ok(())
}
