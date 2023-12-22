mod abi;
mod asset;
mod config;
mod pool;
mod provider;
mod services;
mod split;
mod utils;

use std::sync::Arc;

use axum::{routing::get, Router};
use config::Config;
use pool::Pool;
use provider::Providers;
use shuttle_secrets::{SecretStore, Secrets};
use tower_http::trace::{DefaultOnFailure, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::{level_filters::LevelFilter, Level};
use tracing_subscriber::EnvFilter;

#[shuttle_runtime::main]
async fn axum(#[Secrets] secret_store: SecretStore) -> shuttle_axum::ShuttleAxum {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();

    let config = Config::get();

    let pools = config.pools.iter().map(Pool::from).collect::<Vec<_>>();
    let providers = Providers::try_from(&secret_store).unwrap();

    let router = Router::new()
        .route("/pools/assets", get(services::pools::assets))
        .route("/staking/share", get(services::staking::share))
        .with_state((Arc::new(providers), Arc::new(pools)))
        .layer(
            TraceLayer::new_for_http()
                .on_request(DefaultOnRequest::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO))
                .on_failure(DefaultOnFailure::new()),
        );

    Ok(router.into())
}
