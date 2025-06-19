//! Example Rust-based SBF program that tests rand behavior

#![allow(unreachable_code)]

use {
    gorbagana_account_info::AccountInfo, gorbagana_msg::msg, gorbagana_program_error::ProgramResult,
    gorbagana_pubkey::Pubkey,
};

gorbagana_program_entrypoint::entrypoint_no_alloc!(process_instruction);
fn process_instruction(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _instruction_data: &[u8],
) -> ProgramResult {
    msg!("rand");
    Ok(())
}
