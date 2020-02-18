use std::time::Duration;
use std::sync::Arc;

use hyper::Client;
use futures::future;
use tokio::time::delay_for;
use tokio::sync::RwLock;
use anyhow::{
    Result,
    Context
};
use slog::Logger;
use kube::{
    config,
    client::APIClient
};
use serde::{Deserialize};
use schemars::JsonSchema;

use super::ServiceCoordinates;
use super::kubernetes;
use super::protocol::SingularEndpoint;

#[derive(Clone, Debug, Deserialize, JsonSchema)]
pub struct DiscoveryOptions {
    pub interval_secs: u32,
    pub consecutive_error_threshold: u32,
    pub svc_coords: ServiceCoordinates,
    pub query_path: String,
}

pub async fn run_endpoint_discovery(opts: DiscoveryOptions, log: Logger, forward_uri_lock: Arc<RwLock<String>>) -> Result<()> {
    let config = config::load_kube_config().await
        .map_err(kubernetes::to_anyhow)
        .context("failed loading kube config")?;
    let client = APIClient::new(config);

    info!(log, "singular running");

    let mut consecutive_err_count: u32 = 0;
    loop {
        debug!(log, "retrieving endpoint...");

        let res = get_singular_endpoint(client.clone(), opts.clone(), log.clone()).await;
        match res {
            Err(e) => {
                consecutive_err_count += 1;
                if consecutive_err_count > opts.consecutive_error_threshold {
                    return Err(e).context("failed retrieving singular endpoint");
                }

                warn!(log, "failed retrieving singular endpoint: {}", e);
            },
            Ok(endpoint) => {
                consecutive_err_count = 0;

                // notify proxy about new endpoint (if it changed at all)
                let new_endpoint_uri = endpoint.to_string();
                let old_endpoint_uri = { forward_uri_lock.read().await };
                if new_endpoint_uri == *old_endpoint_uri {
                    let mut w = forward_uri_lock.write().await;
                    *w = new_endpoint_uri;
                }
            }
        };
        
        let interval = Duration::from_secs(opts.interval_secs.into());
        delay_for(interval).await;
    }
}


async fn get_singular_endpoint(client: APIClient, opts: DiscoveryOptions, log: Logger) -> anyhow::Result<SingularEndpoint> {
    let endpoints = kubernetes::get_service_endpoint(client, &opts.svc_coords).await?;

    // Query all endpoints if they think they are the one
    let singular_queries = endpoints.iter()
        .map(|e| query_singular_endpoint(e, &opts.query_path));
    let singular_results = future::join_all(singular_queries).await;

    // Ignore failed queries
    let responses: Vec<&(SingularEndpoint, bool)> = singular_results.iter()
        .filter(|res| res.is_ok())
        .map(|res| res.as_ref().unwrap())
        .collect();
    
    // Sanity: Any responses at all?
    if responses.is_empty() {
        return Err(anyhow!("0/{} singular queries successful.", endpoints.len()));
    }

    // Filter by positive responses, e.g. which endpoints deemed themselves in charge
    let positive_response: Vec<&(SingularEndpoint, bool)> = responses.iter()
        .filter(|r| r.1)
        .cloned()
        .collect();
    match positive_response.len() {
        0 => {
            Err(anyhow!("none of the singular endpoints felt responsible!"))
        },
        1 => {
            let (endpoint, _) = positive_response.first().unwrap();
            Ok(endpoint.clone())
        },
        _ => {
            let all_endpoints_str = positive_response.iter()
                .map(|e| format!("{}", e.0))
                .collect::<Vec<String>>()
                .join(", ");
            warn!(log, "more than 1 singular endpoint felt responsible [{}], chosing first.", all_endpoints_str);

            let (endpoint, _) = positive_response.first().unwrap();
            Ok(endpoint.clone())
        }
    }
}

async fn query_singular_endpoint(endpoint: &SingularEndpoint, path: &str) -> anyhow::Result<(SingularEndpoint, bool)> {
    let uri = format!("http://{}{}", endpoint, path).parse()?;
    let client = Client::new();
    let response = client.get(uri).await?;
    Ok((endpoint.clone(), response.status() == 200))
}
