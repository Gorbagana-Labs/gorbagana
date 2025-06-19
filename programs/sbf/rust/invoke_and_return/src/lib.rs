//! Invokes an instruction and returns the invoke result, the instruction invoked
//! uses the instruction data provided and all the accounts

use {
    gorbagana_account_info::AccountInfo,
    gorbagana_instruction::{AccountMeta, Instruction},
    gorbagana_program::program::invoke,
    gorbagana_program_error::ProgramResult,
    gorbagana_pubkey::Pubkey,
};

gorbagana_program_entrypoint::entrypoint_no_alloc!(process_instruction);
fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let to_call = accounts[0].key;
    let instruction = Instruction {
        accounts: accounts[1..]
            .iter()
            .map(|acc| AccountMeta {
                pubkey: *acc.key,
                is_signer: acc.is_signer,
                is_writable: acc.is_writable,
            })
            .collect(),
        data: instruction_data.to_owned(),
        program_id: *to_call,
    };
    // program id account is not required for invocations if the
    // program id is not one of the instruction account metas.
    invoke(&instruction, &accounts[1..])
}
