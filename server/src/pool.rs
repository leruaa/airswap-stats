use ethers::types::{Address, Chain, U256};
use futures::future::{join_all, BoxFuture};

use crate::{
    asset::Asset,
    config::PoolConfig,
    provider::Provider,
    split::Split,
    utils::{self},
};

pub struct Pool {
    pub chain: Chain,
    pub address: Address,
    pub split: Option<Split>,
    pub assets: Vec<Asset>,
}

impl Pool {
    pub fn new(chain: Chain, address: Address, split: Option<Split>, assets: &[Asset]) -> Self {
        Self {
            chain,
            address,
            split,
            assets: Vec::from(assets),
        }
    }

    pub fn assets_ids(&self) -> Vec<String> {
        self.assets.iter().map(|a| a.id.clone()).collect()
    }

    pub async fn get_balances(&self, provider: &Provider) -> Vec<AssetWithBalances> {
        let futures = self
            .assets
            .iter()
            .map(|a| -> BoxedBalanceOfFuture {
                Box::pin(async {
                    let to_distribute = self.get_to_distribute_balance(a, provider).await;
                    let splits_balance = self.get_0xsplits_balance(a, provider).await;
                    AssetWithBalances::new(
                        a.clone(),
                        to_distribute,
                        splits_balance
                            .unwrap_or_default()
                            .checked_sub(to_distribute.unwrap_or_default()),
                        a.balance_of(self.address, provider).await.unwrap(),
                    )
                })
            })
            .collect::<Vec<BoxedBalanceOfFuture>>();

        join_all(futures).await
    }

    async fn get_to_distribute_balance(&self, asset: &Asset, provider: &Provider) -> Option<U256> {
        match &self.split {
            Some(split) => asset.balance_of(split.account, provider).await,
            None => None,
        }
    }

    async fn get_0xsplits_balance(&self, asset: &Asset, provider: &Provider) -> Option<U256> {
        match &self.split {
            Some(split) => provider
                .get_split(split.main)
                .get_erc20_balance(split.account, asset.address)
                .call()
                .await
                .ok(),
            None => None,
        }
    }
}

impl From<&PoolConfig> for Pool {
    fn from(value: &PoolConfig) -> Self {
        let split = value.split.as_ref().map(Into::into);
        Pool::new(value.chain, value.address, split, &value.assets)
    }
}

pub struct AssetWithBalances(pub Asset, pub BalancesRepartition);

impl AssetWithBalances {
    pub fn new(
        asset: Asset,
        to_distribute: Option<U256>,
        to_withdraw: Option<U256>,
        to_claim: U256,
    ) -> Self {
        Self(
            asset,
            BalancesRepartition::new(to_distribute, to_withdraw, to_claim),
        )
    }

    pub fn get_reward(&self, points: Option<f64>, price: f64) -> Option<f64> {
        match points {
            Some(points) => {
                let share = utils::get_share(points);
                let float_balance = utils::uint_to_float(self.1.to_claim, self.0.decimals);
                Some(utils::get_reward(float_balance, share) * price)
            }
            None => None,
        }
    }
}

pub struct BalancesRepartition {
    pub to_distribute: Option<U256>,
    pub to_withdraw: Option<U256>,
    pub to_claim: U256,
}

impl BalancesRepartition {
    pub fn new(to_distribute: Option<U256>, to_withdraw: Option<U256>, to_claim: U256) -> Self {
        Self {
            to_distribute,
            to_withdraw,
            to_claim,
        }
    }
}

type BoxedBalanceOfFuture<'a> = BoxFuture<'a, AssetWithBalances>;
