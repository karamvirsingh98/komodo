use std::{net::SocketAddr, sync::Arc};

use axum::{
    body::Body,
    extract::ConnectInfo,
    http::{Request, StatusCode},
    middleware::{self, Next},
    response::Response,
    routing::get,
    Router,
};
use helpers::docker::DockerClient;
use types::PeripheryConfig;

use crate::PeripheryConfigExtension;

mod accounts;
mod build;
mod container;
mod git;
mod image;
mod network;
mod stats;

pub fn router(config: PeripheryConfigExtension) -> Router {
    Router::new()
        .route("/health", get(|| async {}))
        .route("/accounts/:account_type", get(accounts::get_accounts))
        .nest("/container", container::router())
        .nest("/network", network::router())
        .nest(
            "/stats",
            stats::router(config.stats_polling_rate.to_string().parse().unwrap()),
        )
        .nest("/git", git::router())
        .nest("/build", build::router())
        .nest("/image", image::router())
        .layer(DockerClient::extension())
        .layer(middleware::from_fn(guard_request))
        .layer(config)
}

async fn guard_request(
    req: Request<Body>,
    next: Next<Body>,
) -> Result<Response, (StatusCode, String)> {
    let ConnectInfo(socket_addr) = req.extensions().get::<ConnectInfo<SocketAddr>>().ok_or((
        StatusCode::UNAUTHORIZED,
        "could not get socket addr of request".to_string(),
    ))?;
    let ip = socket_addr.ip();
    let config = req.extensions().get::<Arc<PeripheryConfig>>().ok_or((
        StatusCode::INTERNAL_SERVER_ERROR,
        "could not get periphery config".to_string(),
    ))?;
    if config.allowed_core_ip.contains(&ip) {
        Ok(next.run(req).await)
    } else {
        Err((
            StatusCode::UNAUTHORIZED,
            "requesting ip not allowed".to_string(),
        ))
    }
}
