//! @brief Example Rust-based BPF program that exercises the sol_remaining_compute_units syscall

use {
    gorbagana_account_info::AccountInfo, gorbagana_msg::msg,
    gorbagana_program::compute_units::sol_remaining_compute_units,
    gorbagana_program_error::ProgramResult, gorbagana_pubkey::Pubkey,
};
gorbagana_program_entrypoint::entrypoint_no_alloc!(process_instruction);
pub fn process_instruction(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _instruction_data: &[u8],
) -> ProgramResult {
    let mut i = 0u32;
    for _ in 0..100_000 {
        if i % 500 == 0 {
            let remaining = sol_remaining_compute_units();
            msg!("remaining compute units: {:?}", remaining);
            if remaining < 25_000 {
                break;
            }
        }
        i = i.saturating_add(1);
    }

    msg!("i: {:?}", i);

    Ok(())
}
