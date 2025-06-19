//! Example Rust-based SBF program that uses sol_log_data syscall

use {
    gorbagana_account_info::AccountInfo,
    gorbagana_program::{log::sol_log_data, program::set_return_data},
    gorbagana_program_error::ProgramResult,
    gorbagana_pubkey::Pubkey,
};

gorbagana_program_entrypoint::entrypoint_no_alloc!(process_instruction);
#[allow(clippy::cognitive_complexity)]
fn process_instruction(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let fields: Vec<&[u8]> = instruction_data.split(|e| *e == 0).collect();

    set_return_data(&[0x08, 0x01, 0x44]);

    sol_log_data(&fields);

    Ok(())
}
