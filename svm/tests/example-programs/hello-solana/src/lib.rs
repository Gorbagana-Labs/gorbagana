use {
    gorbagana_account_info::AccountInfo,
    gorbagana_program_entrypoint::entrypoint,
    gorbagana_program_error::ProgramResult,
    gorbagana_msg::msg,
    gorbagana_pubkey::Pubkey,
};

entrypoint!(process_instruction);

fn process_instruction(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _instruction_data: &[u8],
) -> ProgramResult {
    msg!("Hello, Gorbagana!");

    Ok(())
}
