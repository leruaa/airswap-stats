use crate::asset::Asset;
use ethers::types::{Address, Chain};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
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
    pub assets: Vec<Asset>,
    pub provider_url: String,
    pub address: Address,
}
