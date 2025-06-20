use {
    gorbagana_account_info::AccountInfo, gorbagana_program_entrypoint::entrypoint,
    gorbagana_program_error::ProgramResult, gorbagana_pubkey::Pubkey,
    gorbagana_sysvar::{clock::Clock, Sysvar}, gorbagana_program::program::set_return_data
};

entrypoint!(process_instruction);

fn process_instruction(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _instruction_data: &[u8],
) -> ProgramResult {

    let time_now = Clock::get().unwrap().unix_timestamp;
    let return_data = time_now.to_be_bytes();
    set_return_data(&return_data);
    Ok(())
}
