use ethers::types::{Address, Chain};

use crate::config::SplitConfig;

pub struct Split {
    pub chain: Chain,
    pub source: Address,
    pub account: Address,
    pub main: Address,
}

impl From<&SplitConfig> for Split {
    fn from(value: &SplitConfig) -> Self {
        Self {
            chain: value.chain,
            source: value.source,
            account: value.account,
            main: value.main,
        }
    }
}
