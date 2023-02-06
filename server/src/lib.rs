mod abi;
mod asset;
mod config;
mod pool;
mod prices_feed;
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
use sync_wrapper::SyncWrapper;

#[shuttle_service::main]
async fn axum(#[Secrets] secret_store: SecretStore) -> shuttle_service::ShuttleAxum {
    let config = Config::get();

    let pools = config.pools.iter().map(Pool::from).collect::<Vec<_>>();
    let providers = Providers::try_from(&secret_store).unwrap();

    let router = Router::new()
        .route("/pools/assets", get(services::pools::assets))
        .route("/staking/share", get(services::staking::share))
        .with_state((Arc::new(providers), Arc::new(pools)));

    let sync_wrapper = SyncWrapper::new(router);

    Ok(sync_wrapper)
}
