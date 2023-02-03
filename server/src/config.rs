use crate::asset::Asset;
use ethers::types::{Address, Chain};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub providers: Vec<ProviderConfig>,
    pub pools: Vec<PoolConfig>,
}

impl Config {
    pub fn get() -> Self {
        serde_json::from_str(include_str!("../config.json")).unwrap()
    }
}

#[derive(Debug, Deserialize)]
pub struct PoolConfig {
    pub chain: Chain,
    pub address: Address,
    pub split: Option<SplitConfig>,
    pub assets: Vec<Asset>,
}

#[derive(Debug, Deserialize)]
pub struct SplitConfig {
    pub chain: Chain,
    pub source: Address,
    pub account: Address,
    pub main: Address,
}

#[derive(Debug, Deserialize)]
pub struct ProviderConfig {
    pub chain: Chain,
    pub url: String,
}
