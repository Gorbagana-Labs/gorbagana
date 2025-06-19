//! Example Rust-based SBF noop program

use {
    gorbagana_account_info::AccountInfo, gorbagana_program_error::ProgramResult, gorbagana_pubkey::Pubkey,
};

// This program intentionally uses `entrypoint!` instead of `entrypoint_no_alloc!`
// to handle any number of accounts.
gorbagana_program_entrypoint::entrypoint!(process_instruction);
fn process_instruction(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _instruction_data: &[u8],
) -> ProgramResult {
    Ok(())
}
