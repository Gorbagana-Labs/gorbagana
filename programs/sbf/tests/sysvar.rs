#![cfg(feature = "sbf_rust")]

use {
    agave_feature_set::disable_fees_sysvar,
    gorbagana_instruction::{AccountMeta, Instruction},
    gorbagana_keypair::Keypair,
    gorbagana_message::Message,
    gorbagana_pubkey::Pubkey,
    gorbagana_runtime::{
        bank::Bank,
        bank_client::BankClient,
        genesis_utils::{create_genesis_config, GenesisConfigInfo},
        loader_utils::load_program_of_loader_v4,
    },
    gorbagana_runtime_transaction::runtime_transaction::RuntimeTransaction,
    gorbagana_sdk_ids::sysvar::{
        clock, epoch_schedule, instructions, recent_blockhashes, rent, slot_hashes, slot_history,
        stake_history,
    },
    gorbagana_signer::Signer,
    gorbagana_sysvar::{
        epoch_rewards,
        stake_history::{StakeHistory, StakeHistoryEntry},
    },
    gorbagana_transaction::Transaction,
};

#[test]
fn test_sysvar_syscalls() {
    gorbagana_logger::setup();

    let GenesisConfigInfo {
        mut genesis_config,
        mint_keypair,
        ..
    } = create_genesis_config(50);
    genesis_config.accounts.remove(&disable_fees_sysvar::id());

    let bank = Bank::new_for_tests(&genesis_config);

    let epoch_rewards = epoch_rewards::EpochRewards {
        distribution_starting_block_height: 42,
        total_rewards: 100,
        distributed_rewards: 50,
        active: true,
        ..epoch_rewards::EpochRewards::default()
    };
    bank.set_sysvar_for_tests(&epoch_rewards);

    let stake_history = {
        let mut stake_history = StakeHistory::default();
        stake_history.add(
            0,
            StakeHistoryEntry {
                effective: 200,
                activating: 300,
                deactivating: 400,
            },
        );
        stake_history
    };
    bank.set_sysvar_for_tests(&stake_history);

    let (bank, bank_forks) = bank.wrap_with_bank_forks_for_tests();
    let mut bank_client = BankClient::new_shared(bank);
    let authority_keypair = Keypair::new();
    let (bank, program_id) = load_program_of_loader_v4(
        &mut bank_client,
        bank_forks.as_ref(),
        &mint_keypair,
        &authority_keypair,
        "gorbagana_sbf_rust_sysvar",
    );
    bank.freeze();

    for ix_discriminator in 0..4 {
        let instruction = Instruction::new_with_bincode(
            program_id,
            &[ix_discriminator],
            vec![
                AccountMeta::new(mint_keypair.pubkey(), true),
                AccountMeta::new(Pubkey::new_unique(), false),
                AccountMeta::new_readonly(clock::id(), false),
                AccountMeta::new_readonly(epoch_schedule::id(), false),
                AccountMeta::new_readonly(instructions::id(), false),
                #[allow(deprecated)]
                AccountMeta::new_readonly(recent_blockhashes::id(), false),
                AccountMeta::new_readonly(rent::id(), false),
                AccountMeta::new_readonly(slot_hashes::id(), false),
                AccountMeta::new_readonly(slot_history::id(), false),
                AccountMeta::new_readonly(stake_history::id(), false),
                AccountMeta::new_readonly(epoch_rewards::id(), false),
            ],
        );
        let blockhash = bank.last_blockhash();
        let message = Message::new(&[instruction], Some(&mint_keypair.pubkey()));
        let transaction = Transaction::new(&[&mint_keypair], message, blockhash);
        let sanitized_tx = RuntimeTransaction::from_transaction_for_tests(transaction);
        let result = bank.simulate_transaction(&sanitized_tx, false);
        assert!(result.result.is_ok());
    }
}
