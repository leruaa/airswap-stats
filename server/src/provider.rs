use std::{collections::HashMap, sync::Arc};

use crate::{
    abi::{Erc20Contract, SplitContract},
    asset::Asset,
    config::{Config, ProviderConfig},
};
use ethers::{
    abi::Address,
    providers::{Http, Provider as EthersProvider},
    types::Chain,
};
use parking_lot::Mutex;

type Erc20Map = HashMap<Address, Arc<Erc20Contract<EthersProvider<Http>>>>;
type SplitOption = Option<Arc<SplitContract<EthersProvider<Http>>>>;

pub struct Provider {
    inner: Arc<EthersProvider<Http>>,
    erc20_contracts: Arc<Mutex<Erc20Map>>,
    split_contract: Arc<Mutex<SplitOption>>,
}

impl Provider {
    pub fn new(inner: EthersProvider<Http>) -> Self {
        let inner = Arc::new(inner);
        Self {
            inner,
            erc20_contracts: Arc::new(Mutex::new(HashMap::new())),
            split_contract: Arc::new(Mutex::new(None)),
        }
    }

    pub fn get_erc20(&self, asset: &Asset) -> Arc<Erc20Contract<EthersProvider<Http>>> {
        self.erc20_contracts
            .lock()
            .entry(asset.address)
            .or_insert_with(|| Arc::new(Erc20Contract::new(asset.address, self.inner.clone())))
            .clone()
    }

    pub fn get_split(&self, address: Address) -> Arc<SplitContract<EthersProvider<Http>>> {
        self.split_contract
            .lock()
            .get_or_insert_with(|| Arc::new(SplitContract::new(address, self.inner.clone())))
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
