use anyhow::Result;
use kube::api::Api;
use kube::client::APIClient;

use super::protocol::{
    SingularEndpoint,
    ServiceCoordinates
};

pub async fn get_service_endpoint(client: APIClient, service_coords: &ServiceCoordinates) -> Result<Vec<SingularEndpoint>> {
    let endpoints_api = Api::v1Endpoint(client)
        .within(&service_coords.namespace);
    let endpoint = endpoints_api.get(&service_coords.name)
        .await
        .map_err(to_anyhow)?;
    
    let mut result: Option<Vec<SingularEndpoint>> = None;
    for subset in endpoint.subsets {
        let (addrs, ports) = match (subset.addresses, subset.ports) {
            (Some(a), Some(p)) => (a, p),
            _ => continue,
        };
        if addrs.is_empty() {
            continue;
        }

        let port = match ports.iter().find(|&p| p.port == service_coords.port) {
            Some(p) => p,
            None => continue,
        };

        let endpoints = addrs.iter()
            .map(|a| SingularEndpoint{
                ip: a.ip.clone(),
                port: port.port,
            })
            .collect();
        result.replace(endpoints);
        break;
    }
    result.ok_or_else(|| anyhow!("Could not find any Endpoint matching port and service name!"))
}

pub fn to_anyhow(kube_err: kube::Error) -> anyhow::Error {
    anyhow!("{}", kube_err)
}
