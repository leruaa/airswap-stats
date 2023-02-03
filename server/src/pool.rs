use ethers::types::{Address, Chain, U256};
use futures::future::{join_all, BoxFuture};

use crate::{
    asset::Asset,
    config::PoolConfig,
    provider::Provider,
    utils::{self},
};

pub struct Pool {
    pub chain: Chain,
    pub address: Address,
    pub assets: Vec<Asset>,
}

impl Pool {
    pub fn new(chain: Chain, address: Address, assets: &[Asset]) -> Self {
        Self {
            chain,
            address,
            assets: Vec::from(assets),
        }
    }

    pub fn assets_ids(&self) -> Vec<String> {
        self.assets.iter().map(|a| a.id.clone()).collect()
    }

    pub async fn balance_of(&self, provider: &Provider) -> Vec<AssetWithBalance> {
        let futures = self
            .assets
            .iter()
            .map(|a| -> BoxedBalanceOfFuture {
                Box::pin(self.get_asset_balance(a, provider.clone()))
            })
            .collect::<Vec<BoxedBalanceOfFuture>>();

        join_all(futures).await
    }

    async fn get_asset_balance(&self, asset: &Asset, provider: &Provider) -> AssetWithBalance {
        let amount = provider
            .get_erc20(asset)
            .balance_of(self.address)
            .call()
            .await
            .unwrap();

        AssetWithBalance(asset.clone(), amount)
    }
}

impl From<&PoolConfig> for Pool {
    fn from(value: &PoolConfig) -> Self {
        Pool::new(value.chain, value.address, &value.assets)
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
