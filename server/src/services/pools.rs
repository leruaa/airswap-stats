use std::{collections::HashSet, sync::Arc};

use axum::{
    extract::{Query, State},
    Json,
};
use futures::{future::join_all, FutureExt};
use serde::{Deserialize, Serialize};

use crate::{
    pool::Pool,
    prices::{BinancePriceFeed, PriceFeed},
    provider::Providers,
    utils::uint_to_float,
};

type AssetsState = (Arc<Providers>, Arc<Vec<Pool>>);

#[derive(Debug, Clone, Deserialize)]
pub struct QueryParams {
    points: Option<f64>,
}

pub async fn assets(
    State((providers, pools)): State<AssetsState>,
    Query(params): Query<QueryParams>,
) -> Result<Json<Vec<PoolHoldings>>, String> {
    let prices_feed = BinancePriceFeed::new();
    let assets = pools
        .iter()
        .flat_map(|p| p.assets().cloned())
        .collect::<HashSet<_>>();

    let prices = prices_feed.get_prices(assets).await;

    let mut assets_balances = join_all(
        pools
            .iter()
            .filter_map(|p| providers.get(&p.chain).map(|provider| (p, provider)))
            .map(|(pool, provider)| {
                pool.get_balances(provider)
                    .map(|a| a.into_iter().map(|a| (pool.chain, a)).collect::<Vec<_>>())
            })
            .collect::<Vec<_>>(),
    )
    .await
    .iter()
    .flatten()
    .filter_map(|(chain, asset)| {
        prices
            .get(&asset.0.id)
            .map(|p| (chain, asset, p.as_ref().unwrap_or(&0_f64)))
    })
    .map(|(chain, asset, price)| {
        PoolHoldings::new(
            chain.to_string(),
            asset.0.ticker.clone(),
            *price,
            asset
                .1
                .to_distribute
                .map(|v| uint_to_float(v, asset.0.decimals)),
            asset
                .1
                .to_withdraw
                .map(|v| uint_to_float(v, asset.0.decimals)),
            uint_to_float(asset.1.to_claim, asset.0.decimals),
            asset.get_reward(params.points, *price),
        )
    })
    .collect::<Vec<_>>();

    assets_balances.sort();
    assets_balances.reverse();

    Ok(Json(assets_balances))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PoolHoldings {
    chain: String,
    ticker: String,
    usd_price: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    to_distribute: Option<BalanceAndUsdValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    to_withdraw: Option<BalanceAndUsdValue>,
    to_claim: BalanceAndUsdValue,
    total: BalanceAndUsdValue,
    #[serde(skip_serializing_if = "Option::is_none")]
    claimable_usd_value: Option<f64>,
}

impl PoolHoldings {
    pub fn new(
        chain: String,
        ticker: String,
        usd_price: f64,
        balance_to_distribute: Option<f64>,
        balance_to_withdraw: Option<f64>,
        balance_to_claim: f64,
        claimable_usd_value: Option<f64>,
    ) -> Self {
        Self {
            chain,
            ticker,
            usd_price,
            to_distribute: balance_to_distribute.map(|b| BalanceAndUsdValue::new(b, usd_price)),
            to_withdraw: balance_to_withdraw.map(|b| BalanceAndUsdValue::new(b, usd_price)),
            to_claim: BalanceAndUsdValue::new(balance_to_claim, usd_price),
            total: BalanceAndUsdValue::new(
                balance_to_distribute.unwrap_or_default()
                    + balance_to_withdraw.unwrap_or_default()
                    + balance_to_claim,
                usd_price,
            ),
            claimable_usd_value,
        }
    }
}

impl Eq for PoolHoldings {}

impl PartialEq for PoolHoldings {
    fn eq(&self, other: &Self) -> bool {
        self.ticker == other.ticker
            && self.total == other.total
            && self.claimable_usd_value == other.claimable_usd_value
    }
}

impl Ord for PoolHoldings {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.total < other.total {
            std::cmp::Ordering::Less
        } else if self.total > other.total {
            std::cmp::Ordering::Greater
        } else {
            std::cmp::Ordering::Equal
        }
    }
}

impl PartialOrd for PoolHoldings {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.total.partial_cmp(&other.total)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BalanceAndUsdValue {
    balance: f64,
    usd_value: f64,
}

impl BalanceAndUsdValue {
    pub fn new(balance: f64, usd_price: f64) -> Self {
        Self {
            balance,
            usd_value: balance * usd_price,
        }
    }
}

impl PartialEq for BalanceAndUsdValue {
    fn eq(&self, other: &Self) -> bool {
        self.usd_value == other.usd_value
    }
}

impl PartialOrd for BalanceAndUsdValue {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.usd_value.partial_cmp(&other.usd_value)
    }
}
