use ethers::types::{Address, Chain};

use crate::config::SplitConfig;

pub struct Split {
    pub chain: Chain,
    pub account: Address,
    pub main: Address,
}

impl From<&SplitConfig> for Split {
    fn from(value: &SplitConfig) -> Self {
        Self {
            chain: value.chain,
            account: value.account,
            main: value.main,
        }
    }
}
