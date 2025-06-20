//! Example/test program to trigger vm error by dividing by zero

#![feature(asm_experimental_arch)]

use {
    gorbagana_account_info::AccountInfo, gorbagana_program_error::ProgramResult, gorbagana_pubkey::Pubkey,
    std::arch::asm,
};

gorbagana_program_entrypoint::entrypoint_no_alloc!(process_instruction);
fn process_instruction(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _instruction_data: &[u8],
) -> ProgramResult {
    unsafe {
        asm!(
            "
            mov64 r0, 0
            mov64 r1, 0
            div64 r0, r1
        "
        );
    }
    Ok(())
}
