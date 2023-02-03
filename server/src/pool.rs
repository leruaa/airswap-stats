use std::{collections::HashMap, sync::Arc};

use ethers::{
    providers::{Http, Provider},
    types::{Address, Chain, U256},
};
use futures::future::{join_all, BoxFuture};

use crate::{
    abi::Erc20Contract,
    asset::Asset,
    config::PoolConfig,
    utils::{self},
};

pub struct Pool {
    pub chain: Chain,
    pub address: Address,
    pub assets: Vec<Asset>,
    erc20_contracts: HashMap<String, Erc20Contract<Provider<Http>>>,
}

impl Pool {
    pub fn new(chain: Chain, address: Address, provider: Provider<Http>, assets: &[Asset]) -> Self {
        let provider = Arc::new(provider);
        let erc20_contracts = assets
            .iter()
            .map(|a| {
                (
                    a.id.clone(),
                    Erc20Contract::new(a.address, provider.clone()),
                )
            })
            .collect::<HashMap<_, _>>();

        Self {
            chain,
            address,
            assets: Vec::from(assets),
            erc20_contracts,
        }
    }

    pub fn asset_ids(&self) -> Vec<String> {
        self.erc20_contracts.keys().cloned().collect::<Vec<_>>()
    }

    pub async fn balance_of(&self) -> Vec<AssetWithBalance> {
        let futures = self
            .assets
            .iter()
            .map(|a| -> BoxedBalanceOfFuture { Box::pin(self.get_asset_balance(a)) })
            .collect::<Vec<BoxedBalanceOfFuture>>();

        join_all(futures).await
    }

    async fn get_asset_balance(&self, asset: &Asset) -> AssetWithBalance {
        let amount = self
            .erc20_contracts
            .get(&asset.id)
            .unwrap()
            .balance_of(self.address)
            .call()
            .await
            .unwrap();

        AssetWithBalance(asset.clone(), amount)
    }
}

impl From<&PoolConfig> for Pool {
    fn from(value: &PoolConfig) -> Self {
        let provider = Provider::<Http>::try_from(&value.provider_url).unwrap();

        Pool::new(value.chain, value.address, provider, &value.assets)
    }
}

pub struct AssetWithBalance(pub Asset, pub U256);

impl AssetWithBalance {
    pub fn get_reward(&self, points: Option<f64>, price: f64) -> Option<f64> {
        match points {
            Some(points) => {
                let share = utils::get_share(points);
                let float_balance = utils::uint_to_float(self.1, self.0.decimals);
                Some(utils::get_reward(float_balance, share) * price)
            }
            None => None,
        }
    }
}

type BoxedBalanceOfFuture<'a> = BoxFuture<'a, AssetWithBalance>;
