#![cfg(feature = "dev-context-only-utils")]
use {
    crate::{bank::Bank, bank_client::BankClient, bank_forks::BankForks},
    serde::Serialize,
    gorbagana_account::{AccountSharedData, WritableAccount},
    gorbagana_client_traits::{Client, SyncClient},
    gorbagana_clock::Clock,
    gorbagana_instruction::{AccountMeta, Instruction},
    gorbagana_keypair::Keypair,
    gorbagana_loader_v3_interface::state::UpgradeableLoaderState,
    gorbagana_loader_v4_interface::instruction,
    gorbagana_message::Message,
    gorbagana_pubkey::Pubkey,
    gorbagana_sdk_ids::loader_v4,
    gorbagana_signer::Signer,
    gorbagana_system_interface::instruction as system_instruction,
    std::{
        env,
        fs::File,
        io::Read,
        path::PathBuf,
        sync::{Arc, RwLock},
    },
};

const CHUNK_SIZE: usize = 512; // Size of chunk just needs to fit into tx

pub fn load_program_from_file(name: &str) -> Vec<u8> {
    let mut pathbuf = {
        let current_exe = env::current_exe().unwrap();
        PathBuf::from(
            current_exe
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .parent()
                .unwrap(),
        )
    };
    pathbuf.push("deploy");
    pathbuf.push(name);
    pathbuf.set_extension("so");
    let mut file = File::open(&pathbuf).unwrap_or_else(|err| {
        panic!("Failed to open {}: {}", pathbuf.display(), err);
    });
    let mut program = Vec::new();
    file.read_to_end(&mut program).unwrap();
    program
}

// Creates an unverified program by bypassing the loader built-in program
pub fn create_program(bank: &Bank, loader_id: &Pubkey, name: &str) -> Pubkey {
    let program_id = Pubkey::new_unique();
    let elf = load_program_from_file(name);
    let mut program_account = AccountSharedData::new(1, elf.len(), loader_id);
    program_account
        .data_as_mut_slice()
        .get_mut(..)
        .unwrap()
        .copy_from_slice(&elf);
    program_account.set_executable(true);
    bank.store_account(&program_id, &program_account);
    program_id
}

pub fn load_upgradeable_buffer<T: Client>(
    bank_client: &T,
    from_keypair: &Keypair,
    buffer_keypair: &Keypair,
    buffer_authority_keypair: &Keypair,
    name: &str,
) -> Vec<u8> {
    let program = load_program_from_file(name);
    let buffer_pubkey = buffer_keypair.pubkey();
    let buffer_authority_pubkey = buffer_authority_keypair.pubkey();
    let program_buffer_bytes = UpgradeableLoaderState::size_of_buffer(program.len());

    bank_client
        .send_and_confirm_message(
            &[from_keypair, buffer_keypair],
            Message::new(
                &gorbagana_loader_v3_interface::instruction::create_buffer(
                    &from_keypair.pubkey(),
                    &buffer_pubkey,
                    &buffer_authority_pubkey,
                    1.max(
                        bank_client
                            .get_minimum_balance_for_rent_exemption(program_buffer_bytes)
                            .unwrap(),
                    ),
                    program.len(),
                )
                .unwrap(),
                Some(&from_keypair.pubkey()),
            ),
        )
        .unwrap();

    let chunk_size = CHUNK_SIZE;
    let mut offset = 0;
    for chunk in program.chunks(chunk_size) {
        let message = Message::new(
            &[gorbagana_loader_v3_interface::instruction::write(
                &buffer_pubkey,
                &buffer_authority_pubkey,
                offset,
                chunk.to_vec(),
            )],
            Some(&from_keypair.pubkey()),
        );
        bank_client
            .send_and_confirm_message(&[from_keypair, buffer_authority_keypair], message)
            .unwrap();
        offset += chunk_size as u32;
    }

    program
}

#[deprecated(since = "2.2.0", note = "Use load_program_of_loader_v4() instead")]
pub fn load_upgradeable_program(
    bank_client: &BankClient,
    from_keypair: &Keypair,
    buffer_keypair: &Keypair,
    executable_keypair: &Keypair,
    authority_keypair: &Keypair,
    name: &str,
) {
    let program = load_upgradeable_buffer(
        bank_client,
        from_keypair,
        buffer_keypair,
        authority_keypair,
        name,
    );

    #[allow(deprecated)]
    let message = Message::new(
        &gorbagana_loader_v3_interface::instruction::deploy_with_max_program_len(
            &from_keypair.pubkey(),
            &executable_keypair.pubkey(),
            &buffer_keypair.pubkey(),
            &authority_keypair.pubkey(),
            1.max(
                bank_client
                    .get_minimum_balance_for_rent_exemption(
                        UpgradeableLoaderState::size_of_program(),
                    )
                    .unwrap(),
            ),
            program.len() * 2,
        )
        .unwrap(),
        Some(&from_keypair.pubkey()),
    );
    bank_client
        .send_and_confirm_message(
            &[from_keypair, executable_keypair, authority_keypair],
            message,
        )
        .unwrap();
    bank_client.set_sysvar_for_tests(&Clock {
        slot: 1,
        ..Clock::default()
    });
}

#[deprecated(since = "2.2.0", note = "Use load_program_of_loader_v4() instead")]
pub fn load_upgradeable_program_wrapper(
    bank_client: &BankClient,
    mint_keypair: &Keypair,
    authority_keypair: &Keypair,
    name: &str,
) -> Pubkey {
    let buffer_keypair = Keypair::new();
    let program_keypair = Keypair::new();
    #[allow(deprecated)]
    load_upgradeable_program(
        bank_client,
        mint_keypair,
        &buffer_keypair,
        &program_keypair,
        authority_keypair,
        name,
    );
    program_keypair.pubkey()
}

#[deprecated(since = "2.2.0", note = "Use load_program_of_loader_v4() instead")]
pub fn load_upgradeable_program_and_advance_slot(
    bank_client: &mut BankClient,
    bank_forks: &RwLock<BankForks>,
    mint_keypair: &Keypair,
    authority_keypair: &Keypair,
    name: &str,
) -> (Arc<Bank>, Pubkey) {
    #[allow(deprecated)]
    let program_id =
        load_upgradeable_program_wrapper(bank_client, mint_keypair, authority_keypair, name);

    // load_upgradeable_program sets clock sysvar to 1, which causes the program to be effective
    // after 2 slots. They need to be called individually to create the correct fork graph in between.
    bank_client
        .advance_slot(1, bank_forks, &Pubkey::default())
        .expect("Failed to advance the slot");

    let bank = bank_client
        .advance_slot(1, bank_forks, &Pubkey::default())
        .expect("Failed to advance the slot");

    (bank, program_id)
}

pub fn upgrade_program<T: Client>(
    bank_client: &T,
    payer_keypair: &Keypair,
    buffer_keypair: &Keypair,
    executable_pubkey: &Pubkey,
    authority_keypair: &Keypair,
    name: &str,
) {
    load_upgradeable_buffer(
        bank_client,
        payer_keypair,
        buffer_keypair,
        authority_keypair,
        name,
    );
    let message = Message::new(
        &[gorbagana_loader_v3_interface::instruction::upgrade(
            executable_pubkey,
            &buffer_keypair.pubkey(),
            &authority_keypair.pubkey(),
            &payer_keypair.pubkey(),
        )],
        Some(&payer_keypair.pubkey()),
    );
    bank_client
        .send_and_confirm_message(&[payer_keypair, authority_keypair], message)
        .unwrap();
}

pub fn set_upgrade_authority<T: Client>(
    bank_client: &T,
    from_keypair: &Keypair,
    program_pubkey: &Pubkey,
    current_authority_keypair: &Keypair,
    new_authority_pubkey: Option<&Pubkey>,
) {
    let message = Message::new(
        &[
            gorbagana_loader_v3_interface::instruction::set_upgrade_authority(
                program_pubkey,
                &current_authority_keypair.pubkey(),
                new_authority_pubkey,
            ),
        ],
        Some(&from_keypair.pubkey()),
    );
    bank_client
        .send_and_confirm_message(&[from_keypair, current_authority_keypair], message)
        .unwrap();
}

pub fn instructions_to_load_program_of_loader_v4<T: Client>(
    bank_client: &T,
    payer_keypair: &Keypair,
    authority_keypair: &Keypair,
    name: &str,
    program_keypair: Option<Keypair>,
    target_program_id: Option<&Pubkey>,
) -> (Keypair, Vec<Instruction>) {
    let mut instructions = Vec::new();
    let loader_id = &loader_v4::id();
    let program = load_program_from_file(name);
    let program_keypair = program_keypair.unwrap_or_else(|| {
        let program_keypair = Keypair::new();
        instructions.push(system_instruction::create_account(
            &payer_keypair.pubkey(),
            &program_keypair.pubkey(),
            bank_client
                .get_minimum_balance_for_rent_exemption(
                    gorbagana_loader_v4_interface::state::LoaderV4State::program_data_offset()
                        .saturating_add(program.len()),
                )
                .unwrap(),
            0,
            loader_id,
        ));
        program_keypair
    });
    instructions.push(instruction::set_program_length(
        &program_keypair.pubkey(),
        &authority_keypair.pubkey(),
        program.len() as u32,
        &payer_keypair.pubkey(),
    ));
    let chunk_size = CHUNK_SIZE;
    let mut offset = 0;
    for chunk in program.chunks(chunk_size) {
        instructions.push(instruction::write(
            &program_keypair.pubkey(),
            &authority_keypair.pubkey(),
            offset,
            chunk.to_vec(),
        ));
        offset += chunk_size as u32;
    }
    if let Some(target_program_id) = target_program_id {
        instructions.push(instruction::set_program_length(
            target_program_id,
            &authority_keypair.pubkey(),
            program.len() as u32,
            &payer_keypair.pubkey(),
        ));
        instructions.push(instruction::copy(
            target_program_id,
            &authority_keypair.pubkey(),
            &program_keypair.pubkey(),
            0,
            0,
            program.len() as u32,
        ));
        instructions.push(instruction::deploy(
            target_program_id,
            &authority_keypair.pubkey(),
        ));
    } else {
        instructions.push(instruction::deploy(
            &program_keypair.pubkey(),
            &authority_keypair.pubkey(),
        ));
    }
    (program_keypair, instructions)
}

pub fn load_program_of_loader_v4(
    bank_client: &mut BankClient,
    bank_forks: &RwLock<BankForks>,
    payer_keypair: &Keypair,
    authority_keypair: &Keypair,
    name: &str,
) -> (Arc<Bank>, Pubkey) {
    let (program_keypair, instructions) = instructions_to_load_program_of_loader_v4(
        bank_client,
        payer_keypair,
        authority_keypair,
        name,
        None,
        None,
    );
    let signers: &[&[&Keypair]] = &[
        &[payer_keypair, &program_keypair],
        &[payer_keypair, authority_keypair],
    ];
    let signers = std::iter::once(signers[0]).chain(std::iter::repeat(signers[1]));
    for (instruction, signers) in instructions.into_iter().zip(signers) {
        let message = Message::new(&[instruction], Some(&payer_keypair.pubkey()));
        bank_client
            .send_and_confirm_message(signers, message)
            .unwrap();
    }
    let bank = bank_client
        .advance_slot(1, bank_forks, &Pubkey::default())
        .expect("Failed to advance the slot");
    (bank, program_keypair.pubkey())
}

// Return an Instruction that invokes `program_id` with `data` and required
// a signature from `from_pubkey`.
pub fn create_invoke_instruction<T: Serialize>(
    from_pubkey: Pubkey,
    program_id: Pubkey,
    data: &T,
) -> Instruction {
    let account_metas = vec![AccountMeta::new(from_pubkey, true)];
    Instruction::new_with_bincode(program_id, data, account_metas)
}
