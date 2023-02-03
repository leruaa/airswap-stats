use std::collections::HashMap;

use reqwest::Client;
use serde::{Deserialize, Serialize};

pub struct PricesFeed {
    client: Client,
}

#[derive(Debug, Serialize, Deserialize)]
struct Price {
    usd: f64,
}

impl PricesFeed {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub async fn get_prices(&self, asset_ids: &[String]) -> HashMap<String, f64> {
        let asset_ids = asset_ids.join(",");

        let response = self
            .client
            .get("https://api.coingecko.com/api/v3/simple/price")
            .query(&[("ids", asset_ids.as_str()), ("vs_currencies", "usd")])
            .send()
            .await
            .unwrap();

        response
            .json::<HashMap<String, Price>>()
            .await
            .unwrap()
            .into_iter()
            .map(|(k, v)| (k, v.usd))
            .collect::<HashMap<_, _>>()
    }
}
