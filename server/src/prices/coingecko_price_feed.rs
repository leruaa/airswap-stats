use std::collections::{HashMap, HashSet};

use async_trait::async_trait;
use futures::TryFutureExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::asset::Asset;

use super::price_feed::PriceFeed;

pub struct CoinGeckoPriceFeed {
    client: Client,
}

#[derive(Debug, Serialize, Deserialize)]
struct Price {
    usd: f64,
}

impl CoinGeckoPriceFeed {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

#[async_trait]
impl PriceFeed for CoinGeckoPriceFeed {
    async fn get_prices(&self, assets: HashSet<Asset>) -> HashMap<String, Result<f64, String>> {
        let asset_ids = assets
            .into_iter()
            .map(|a| a.id)
            .collect::<Vec<_>>()
            .join(",");

        self.client
            .get("https://api.coingecko.com/api/v3/simple/price")
            .query(&[("ids", asset_ids.as_str()), ("vs_currencies", "usd")])
            .send()
            .and_then(|resp| {
                resp.json::<HashMap<String, Price>>().map_ok(|x| {
                    x.into_iter()
                        .map(|(k, v)| (k, Ok(v.usd)))
                        .collect::<HashMap<_, _>>()
                })
            })
            .map_err(|err| err.to_string())
            .await
            .unwrap()
    }
}
