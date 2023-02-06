use std::{collections::HashMap, sync::Arc};

use crate::{
    abi::{Erc20Contract, SplitContract},
    asset::Asset,
};
use ethers::{
    abi::Address,
    providers::{Http, Provider as EthersProvider},
    types::Chain,
};
use parking_lot::Mutex;
use shuttle_secrets::SecretStore;

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

pub struct Providers(HashMap<Chain, Provider>);

impl Providers {
    pub fn get(&self, chain: &Chain) -> Option<&Provider> {
        self.0.get(chain)
    }
}

impl TryFrom<String> for Provider {
    type Error = String;

    fn try_from(url: String) -> Result<Self, Self::Error> {
        EthersProvider::try_from(url)
            .map(Provider::new)
            .map_err(|err| err.to_string())
    }
}

impl TryFrom<&SecretStore> for Providers {
    type Error = String;

    fn try_from(store: &SecretStore) -> Result<Self, Self::Error> {
        let providers = HashMap::from([
            (
                Chain::Mainnet,
                store
                    .get("ETH_PROVIDER")
                    .ok_or_else(|| String::from("ETH_PROVIDER secret not found"))?
                    .try_into()?,
            ),
            (
                Chain::BinanceSmartChain,
                store
                    .get("BSC_PROVIDER")
                    .ok_or_else(|| String::from("BSC_PROVIDER secret not found"))?
                    .try_into()?,
            ),
        ]);

        Ok(Providers(providers))
    }
}
