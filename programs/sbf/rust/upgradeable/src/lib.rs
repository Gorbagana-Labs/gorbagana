//! Example Rust-based SBF upgradeable program

use {
    gorbagana_account_info::AccountInfo, gorbagana_msg::msg, gorbagana_program_error::ProgramResult,
    gorbagana_pubkey::Pubkey, gorbagana_sysvar::clock,
};

gorbagana_program_entrypoint::entrypoint_no_alloc!(process_instruction);
fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    _instruction_data: &[u8],
) -> ProgramResult {
    msg!("Upgradeable program");
    assert_eq!(accounts.len(), 1);
    assert_eq!(*accounts[0].key, clock::id());
    Err(42.into())
}
