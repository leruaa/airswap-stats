use std::collections::{HashMap, HashSet};

use async_trait::async_trait;
use futures::{future::join_all, FutureExt, TryFutureExt};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::asset::Asset;

use super::price_feed::PriceFeed;

pub struct BinancePriceFeed {
    client: Client,
    ticker_to_symbol: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Price {
    usd: f64,
}

impl BinancePriceFeed {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            ticker_to_symbol: HashMap::from([
                ("WBTC".to_string(), "BTCUSDT".to_string()),
                ("WETH".to_string(), "ETHUSDT".to_string()),
                ("WBNB".to_string(), "BNBUSDT".to_string()),
                ("USDT".to_string(), "USDCUSDT".to_string()),
            ]),
        }
    }

    async fn avg_price(&self, assets: HashSet<Asset>) -> HashMap<String, Result<f64, String>> {
        let request_futures = assets
            .into_iter()
            .map(|a| {
                (
                    a.id,
                    self.ticker_to_symbol
                        .get(&a.ticker)
                        .cloned()
                        .unwrap_or_else(|| format!("{}USDT", a.ticker)),
                )
            })
            .map(|(id, symbol)| {
                self.client
                    .get("https://data.binance.com/api/v3/avgPrice")
                    .query(&[("symbol", symbol.as_str())])
                    .send()
                    .map_err(|err| err.to_string())
                    .and_then(|r| r.json::<AvgPriceResponse>().map_err(|err| err.to_string()))
                    .map(|p| {
                        (
                            id,
                            p.and_then(|p| p.price.parse::<f64>().map_err(|err| err.to_string())),
                        )
                    })
            })
            .collect::<Vec<_>>();

        join_all(request_futures).await.into_iter().collect()
    }
}

#[async_trait]
impl PriceFeed for BinancePriceFeed {
    async fn get_prices(&self, assets: HashSet<Asset>) -> HashMap<String, Result<f64, String>> {
        self.avg_price(assets).await
    }
}

#[derive(Debug, Clone, Deserialize)]
struct AvgPriceResponse {
    mins: u8,
    price: String,
}
