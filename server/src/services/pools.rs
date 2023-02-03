use std::{collections::HashMap, sync::Arc};

use axum::{
    extract::{Query, State},
    Json,
};
use ethers::types::Chain;
use futures::{future::join_all, FutureExt};
use serde::{Deserialize, Serialize};

use crate::{pool::Pool, prices_feed::PricesFeed, provider::Provider, utils::uint_to_float};

#[derive(Debug, Clone, Deserialize)]
pub struct QueryParams {
    points: Option<f64>,
}

pub async fn assets(
    State((providers, pools)): State<(Arc<HashMap<Chain, Provider>>, Arc<Vec<Pool>>)>,
    Query(params): Query<QueryParams>,
) -> Json<Vec<PoolHoldings>> {
    let prices_feed = PricesFeed::new();
    let assets = pools
        .iter()
        .flat_map(|p| p.assets_ids())
        .collect::<Vec<_>>();

    let prices = prices_feed.get_prices(&assets).await;

    let mut assets_balances = join_all(
        pools
            .iter()
            .filter_map(|p| providers.get(&p.chain).map(|provider| (p, provider)))
            .map(|(pool, provider)| {
                pool.balance_of(provider)
                    .map(|a| a.into_iter().map(|a| (pool.chain, a)).collect::<Vec<_>>())
            })
            .collect::<Vec<_>>(),
    )
    .await
    .iter()
    .flatten()
    .filter_map(|(chain, asset)| prices.get(&asset.0.id).map(|p| (chain, asset, p)))
    .map(|(chain, asset, price)| {
        PoolHoldings::new(
            chain.to_string(),
            asset.0.ticker.clone(),
            uint_to_float(asset.1, asset.0.decimals),
            *price,
            asset.get_reward(params.points, *price),
        )
    })
    .collect::<Vec<_>>();

    assets_balances.sort();
    assets_balances.reverse();

    Json(assets_balances)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PoolHoldings {
    chain: String,
    ticker: String,
    balance: f64,
    usd_price: f64,
    usd_value: f64,
    claimable_usd_value: Option<f64>,
}

impl PoolHoldings {
    pub fn new(
        chain: String,
        ticker: String,
        balance: f64,
        usd_price: f64,
        claimable_usd_value: Option<f64>,
    ) -> Self {
        Self {
            chain,
            ticker,
            balance,
            usd_price,
            usd_value: balance * usd_price,
            claimable_usd_value,
        }
    }
}

impl Eq for PoolHoldings {}

impl PartialEq for PoolHoldings {
    fn eq(&self, other: &Self) -> bool {
        self.ticker == other.ticker
            && self.usd_value == other.usd_value
            && self.claimable_usd_value == other.claimable_usd_value
    }
}

impl Ord for PoolHoldings {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.usd_value < other.usd_value {
            std::cmp::Ordering::Less
        } else if self.usd_value > other.usd_value {
            std::cmp::Ordering::Greater
        } else {
            std::cmp::Ordering::Equal
        }
    }
}

impl PartialOrd for PoolHoldings {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.usd_value.partial_cmp(&other.usd_value)
    }
}
