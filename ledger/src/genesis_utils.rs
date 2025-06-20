pub use gorbagana_runtime::genesis_utils::{
    bootstrap_validator_stake_lamports, create_genesis_config_with_leader, GenesisConfigInfo,
};
use {
    gorbagana_keypair::Keypair, gorbagana_pubkey::Pubkey,
    gorbagana_runtime::genesis_utils::create_genesis_config_with_leader_with_mint_keypair,
};

// same as genesis_config::create_genesis_config, but with bootstrap_validator staking logic
//  for the core crate tests
pub fn create_genesis_config(mint_lamports: u64) -> GenesisConfigInfo {
    create_genesis_config_with_leader(
        mint_lamports,
        &Pubkey::new_unique(),
        bootstrap_validator_stake_lamports(),
    )
}

pub fn create_genesis_config_with_mint_keypair(
    mint_keypair: Keypair,
    mint_lamports: u64,
) -> GenesisConfigInfo {
    create_genesis_config_with_leader_with_mint_keypair(
        mint_keypair,
        mint_lamports,
        &Pubkey::new_unique(),
        bootstrap_validator_stake_lamports(),
    )
}
