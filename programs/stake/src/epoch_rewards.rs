//! Creates the initial empty EpochRewards sysvar
use {
    gorbagana_account::{AccountSharedData, WritableAccount},
    gorbagana_genesis_config::GenesisConfig,
    gorbagana_sdk_ids::sysvar,
    gorbagana_sysvar::{
        epoch_rewards::{self, EpochRewards},
        Sysvar,
    },
};

pub fn add_genesis_account(genesis_config: &mut GenesisConfig) -> u64 {
    let data = vec![0; EpochRewards::size_of()];
    let lamports = std::cmp::max(genesis_config.rent.minimum_balance(data.len()), 1);

    let account = AccountSharedData::create(lamports, data, sysvar::id(), false, u64::MAX);

    genesis_config.add_account(epoch_rewards::id(), account);

    lamports
}
