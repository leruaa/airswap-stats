mod abi;
mod asset;
mod config;
mod pool;
mod prices;
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

#[shuttle_runtime::main]
async fn axum(#[Secrets] secret_store: SecretStore) -> shuttle_axum::ShuttleAxum {
    let config = Config::get();

    let pools = config.pools.iter().map(Pool::from).collect::<Vec<_>>();
    let providers = Providers::try_from(&secret_store).unwrap();

    let router = Router::new()
        .route("/pools/assets", get(services::pools::assets))
        .route("/staking/share", get(services::staking::share))
        .with_state((Arc::new(providers), Arc::new(pools)));

    Ok(router.into())
}
