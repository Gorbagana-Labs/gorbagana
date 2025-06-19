//! Example Rust-based SBF program that tests the `sol_get_epoch_stake`
//! syscall.

use {
    gorbagana_account_info::AccountInfo,
    gorbagana_msg::msg,
    gorbagana_program::{
        epoch_stake::{get_epoch_stake_for_vote_account, get_epoch_total_stake},
        program::set_return_data,
    },
    gorbagana_program_error::ProgramResult,
    gorbagana_pubkey::Pubkey,
};

gorbagana_program_entrypoint::entrypoint_no_alloc!(process_instruction);
pub fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    _instruction_data: &[u8],
) -> ProgramResult {
    // Total stake.
    let total_stake = get_epoch_total_stake();
    assert_ne!(total_stake, 0);
    msg!("Total Stake: {}", total_stake);

    // Vote accounts.
    let check_vote_account_stake = |i: usize| {
        let vote_address = accounts[i].key;
        let vote_stake = get_epoch_stake_for_vote_account(vote_address);
        assert_ne!(vote_stake, 0);
        msg!("Vote Stake for account {}: {}", i, vote_stake);
    };
    check_vote_account_stake(0);
    check_vote_account_stake(1);

    // For good measure, set the return data to total stake.
    set_return_data(&total_stake.to_le_bytes());

    Ok(())
}
