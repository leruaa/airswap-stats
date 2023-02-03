use ethers::types::{Address, U256};
use serde::Deserialize;
use std::hash::{Hash, Hasher};

use crate::provider::Provider;

#[derive(Debug, Clone, Deserialize)]
pub struct Asset {
    pub id: String,
    pub ticker: String,
    pub address: Address,
    pub decimals: u8,
}

impl Asset {
    pub async fn balance_of(&self, address: Address, provider: &Provider) -> U256 {
        provider
            .get_erc20(self)
            .balance_of(address)
            .call()
            .await
            .unwrap()
    }
}

impl Hash for Asset {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}
