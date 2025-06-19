//! Example Rust-based SBF program that panics

#[cfg(all(feature = "custom-panic", target_os = "gorbagana"))]
#[no_mangle]
fn custom_panic(info: &core::panic::PanicInfo<'_>) {
    // Note: Full panic reporting is included here for testing purposes
    gorbagana_msg::msg!("program custom panic enabled");
    gorbagana_msg::msg!(&format!("{info}"));
}

use {
    gorbagana_account_info::AccountInfo, gorbagana_program_error::ProgramResult, gorbagana_pubkey::Pubkey,
};

gorbagana_program_entrypoint::entrypoint_no_alloc!(process_instruction);
fn process_instruction(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _instruction_data: &[u8],
) -> ProgramResult {
    assert_eq!(1, 2);
    Ok(())
}
