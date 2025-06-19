//! Example/test program to get the minimum stake delegation via the helper function

#![allow(unreachable_code)]

use {
    gorbagana_account_info::AccountInfo, gorbagana_msg::msg, gorbagana_program_error::ProgramResult,
    gorbagana_pubkey::Pubkey, gorbagana_stake_interface as stake,
};

gorbagana_program_entrypoint::entrypoint_no_alloc!(process_instruction);
fn process_instruction(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _instruction_data: &[u8],
) -> ProgramResult {
    let minimum_delegation = stake::tools::get_minimum_delegation()?;
    msg!(
        "The minimum stake delegation is {} lamports",
        minimum_delegation
    );
    Ok(())
}
