use ethers::types::Address;
use serde::Deserialize;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, Deserialize)]
pub struct Asset {
    pub id: String,
    pub ticker: String,
    pub address: Address,
    pub decimals: u8,
}

impl Hash for Asset {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}
