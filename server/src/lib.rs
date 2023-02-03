mod abi;
mod asset;
mod config;
mod pool;
mod prices_feed;
mod services;
mod utils;

use std::sync::Arc;

use axum::{routing::get, Router};
use config::Config;
use pool::Pool;
use sync_wrapper::SyncWrapper;

#[shuttle_service::main]
async fn axum() -> shuttle_service::ShuttleAxum {
    let config = Config::get();

    let pools = config.pools.iter().map(Pool::from).collect::<Vec<_>>();

    let router = Router::new()
        .route("/pools/assets", get(services::pools::assets))
        .route("/staking/share", get(services::staking::share))
        .with_state(Arc::new(pools));

    let sync_wrapper = SyncWrapper::new(router);

    Ok(sync_wrapper)
}
