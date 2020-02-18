use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;

use tokio::sync::RwLock;
use hyper::server::conn::AddrStream;
use hyper::{Body, Request, Response, Server, Error};
use hyper::service::{service_fn, make_service_fn};

pub async fn run_proxy(local_port: u16, forward_uri_lock: Arc<RwLock<String>>) -> Result<(), Error> {
    let addr: SocketAddr = ([0, 0, 0, 0], local_port).into();

    let svc_fn = make_service_fn(|conn: &AddrStream| {
        let remote_addr = conn.remote_addr();
        let forward_uri_lock = forward_uri_lock.clone();
        async move {
            let svc_fn = service_fn(move |req: Request<Body>| {
                let lock = forward_uri_lock.clone();
                async move {
                    // Don't lock longer than needed (like, for the whole request)
                    let forward_uri = { lock.read().await };
                    let res = hyper_reverse_proxy::call(remote_addr.ip(), &forward_uri, req).await;
                    Ok::<Response<Body>,Infallible>(res)
                }
            });
            Ok::<_, Infallible>(svc_fn)
        }
    });

    Server::bind(&addr)
        .serve(svc_fn)
        .await
}