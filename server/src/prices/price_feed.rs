use std::collections::{HashMap, HashSet};

use async_trait::async_trait;

use crate::asset::Asset;

#[async_trait]
pub trait PriceFeed {
    async fn get_prices(&self, assets: HashSet<Asset>) -> HashMap<String, Result<f64, String>>;
}
