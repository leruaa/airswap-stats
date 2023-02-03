use std::{collections::HashMap, sync::Arc};

use crate::{
    abi::Erc20Contract,
    asset::Asset,
    config::{Config, ProviderConfig},
};
use ethers::{
    abi::Address,
    providers::{Http, Provider as EthersProvider},
    types::Chain,
};
use parking_lot::Mutex;

pub struct Provider {
    inner: Arc<EthersProvider<Http>>,
    erc20_contracts: Arc<Mutex<HashMap<Address, Arc<Erc20Contract<EthersProvider<Http>>>>>>,
}

impl Provider {
    pub fn new(inner: EthersProvider<Http>) -> Self {
        Self {
            inner: Arc::new(inner),
            erc20_contracts: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn get_erc20(&self, asset: &Asset) -> Arc<Erc20Contract<EthersProvider<Http>>> {
        self.erc20_contracts
            .lock()
            .entry(asset.address)
            .or_insert_with(|| Arc::new(Erc20Contract::new(asset.address, self.inner.clone())))
            .clone()
    }
}

impl TryFrom<&ProviderConfig> for Provider {
    type Error = String;

    fn try_from(value: &ProviderConfig) -> Result<Self, Self::Error> {
        let inner = EthersProvider::try_from(value.url.clone()).map_err(|err| err.to_string())?;
        Ok(Provider::new(inner))
    }
}

impl TryFrom<&Config> for HashMap<Chain, Provider> {
    type Error = String;

    fn try_from(config: &Config) -> Result<Self, Self::Error> {
        config
            .providers
            .iter()
            .map(|provider_config| {
                Provider::try_from(provider_config).map(|p| (provider_config.chain, p))
            })
            .collect()
    }
}
