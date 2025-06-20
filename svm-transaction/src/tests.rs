#![cfg(test)]
use {
    crate::svm_message::SVMMessage,
    gorbagana_hash::Hash,
    gorbagana_message::{
        compiled_instruction::CompiledInstruction,
        legacy,
        v0::{self, LoadedAddresses, MessageAddressTableLookup},
        MessageHeader, SanitizedMessage, SanitizedVersionedMessage, SimpleAddressLoader,
        VersionedMessage,
    },
    gorbagana_pubkey::Pubkey,
    gorbagana_sdk_ids::system_program,
    gorbagana_system_interface::instruction::SystemInstruction,
    std::collections::HashSet,
    test_case::test_case,
};

#[test_case(true; "test_get_durable_nonce_static_nonce_account")]
#[test_case(false; "test_get_durable_nonce_alt_nonce_account")]
fn test_get_durable_nonce(require_static_nonce_account: bool) {
    fn create_message_for_test(
        num_signers: u8,
        num_writable: u8,
        account_keys: Vec<Pubkey>,
        instructions: Vec<CompiledInstruction>,
        loaded_addresses: Option<LoadedAddresses>,
    ) -> SanitizedMessage {
        let header = MessageHeader {
            num_required_signatures: num_signers,
            num_readonly_signed_accounts: 0,
            num_readonly_unsigned_accounts: u8::try_from(account_keys.len())
                .unwrap()
                .checked_sub(num_writable)
                .unwrap(),
        };
        let (versioned_message, loader) = match loaded_addresses {
            None => (
                VersionedMessage::Legacy(legacy::Message {
                    header,
                    account_keys,
                    recent_blockhash: Hash::default(),
                    instructions,
                }),
                SimpleAddressLoader::Disabled,
            ),
            Some(loaded_addresses) => (
                VersionedMessage::V0(v0::Message {
                    header,
                    account_keys,
                    recent_blockhash: Hash::default(),
                    instructions,
                    address_table_lookups: vec![MessageAddressTableLookup {
                        account_key: Pubkey::new_unique(),
                        writable_indexes: (0..loaded_addresses.writable.len())
                            .map(|x| x as u8)
                            .collect(),
                        readonly_indexes: (0..loaded_addresses.readonly.len())
                            .map(|x| loaded_addresses.writable.len().saturating_add(x) as u8)
                            .collect(),
                    }],
                }),
                SimpleAddressLoader::Enabled(loaded_addresses),
            ),
        };
        SanitizedMessage::try_new(
            SanitizedVersionedMessage::try_new(versioned_message).unwrap(),
            loader,
            &HashSet::new(),
        )
        .unwrap()
    }

    // No instructions - no nonce
    {
        let message = create_message_for_test(1, 1, vec![Pubkey::new_unique()], vec![], None);
        assert!(SanitizedMessage::get_durable_nonce(&message).is_none());
        assert!(SVMMessage::get_durable_nonce(&message, require_static_nonce_account).is_none());
    }

    // system program id instruction - invalid
    {
        let message = create_message_for_test(
            1,
            1,
            vec![Pubkey::new_unique(), system_program::id()],
            vec![CompiledInstruction::new_from_raw_parts(1, vec![], vec![])],
            None,
        );
        assert!(SanitizedMessage::get_durable_nonce(&message).is_none());
        assert!(SVMMessage::get_durable_nonce(&message, require_static_nonce_account).is_none());
    }

    // system program id instruction - not nonce
    {
        let message = create_message_for_test(
            1,
            1,
            vec![Pubkey::new_unique(), system_program::id()],
            vec![CompiledInstruction::new(
                1,
                &SystemInstruction::Transfer { lamports: 1 },
                vec![0, 0],
            )],
            None,
        );
        assert!(SanitizedMessage::get_durable_nonce(&message).is_none());
        assert!(SVMMessage::get_durable_nonce(&message, require_static_nonce_account).is_none());
    }

    // system program id - nonce instruction (no accounts)
    {
        let message = create_message_for_test(
            1,
            1,
            vec![Pubkey::new_unique(), system_program::id()],
            vec![CompiledInstruction::new(
                1,
                &SystemInstruction::AdvanceNonceAccount,
                vec![],
            )],
            None,
        );
        assert!(SanitizedMessage::get_durable_nonce(&message).is_none());
        assert!(SVMMessage::get_durable_nonce(&message, require_static_nonce_account).is_none());
    }

    // system program id - nonce instruction (non-fee-payer, non-writable)
    {
        let payer = Pubkey::new_unique();
        let nonce = Pubkey::new_unique();
        let message = create_message_for_test(
            1,
            1,
            vec![payer, nonce, system_program::id()],
            vec![CompiledInstruction::new(
                1,
                &SystemInstruction::AdvanceNonceAccount,
                vec![1],
            )],
            None,
        );
        assert!(SanitizedMessage::get_durable_nonce(&message).is_none());
        assert!(SVMMessage::get_durable_nonce(&message, require_static_nonce_account).is_none());
    }

    // system program id - nonce instruction fee-payer
    {
        let payer_nonce = Pubkey::new_unique();
        let message = create_message_for_test(
            1,
            1,
            vec![payer_nonce, system_program::id()],
            vec![CompiledInstruction::new(
                1,
                &SystemInstruction::AdvanceNonceAccount,
                vec![0],
            )],
            None,
        );
        assert_eq!(
            SanitizedMessage::get_durable_nonce(&message),
            Some(&payer_nonce)
        );
        assert_eq!(
            SVMMessage::get_durable_nonce(&message, require_static_nonce_account),
            Some(&payer_nonce)
        );
    }

    // system program id - nonce instruction w/ trailing bytes fee-payer
    {
        let payer_nonce = Pubkey::new_unique();
        let mut instruction_bytes = vec![4, 0, 0, 0];
        instruction_bytes.push(0); // add a trailing byte
        let message = create_message_for_test(
            1,
            1,
            vec![payer_nonce, system_program::id()],
            vec![CompiledInstruction::new_from_raw_parts(
                1,
                instruction_bytes,
                vec![0],
            )],
            None,
        );
        assert_eq!(
            SanitizedMessage::get_durable_nonce(&message),
            Some(&payer_nonce)
        );
        assert_eq!(
            SVMMessage::get_durable_nonce(&message, require_static_nonce_account),
            Some(&payer_nonce)
        );
    }

    // system program id - nonce instruction (non-fee-payer)
    {
        let payer = Pubkey::new_unique();
        let nonce = Pubkey::new_unique();
        let message = create_message_for_test(
            1,
            2,
            vec![payer, nonce, system_program::id()],
            vec![CompiledInstruction::new(
                2,
                &SystemInstruction::AdvanceNonceAccount,
                vec![1],
            )],
            None,
        );
        assert_eq!(SanitizedMessage::get_durable_nonce(&message), Some(&nonce));
        assert_eq!(
            SVMMessage::get_durable_nonce(&message, require_static_nonce_account),
            Some(&nonce)
        );
    }

    // system program id - nonce instruction (non-fee-payer, multiple accounts)
    {
        let payer = Pubkey::new_unique();
        let other = Pubkey::new_unique();
        let nonce = Pubkey::new_unique();
        let message = create_message_for_test(
            1,
            3,
            vec![payer, other, nonce, system_program::id()],
            vec![CompiledInstruction::new(
                3,
                &SystemInstruction::AdvanceNonceAccount,
                vec![2, 1, 0],
            )],
            None,
        );
        assert_eq!(SanitizedMessage::get_durable_nonce(&message), Some(&nonce));
        assert_eq!(
            SVMMessage::get_durable_nonce(&message, require_static_nonce_account),
            Some(&nonce)
        );
    }

    // system program id - nonce instruction (non-fee-payer, loaded account)
    {
        let payer = Pubkey::new_unique();
        let nonce = Pubkey::new_unique();
        let message = create_message_for_test(
            1,
            1,
            vec![payer, system_program::id()],
            vec![CompiledInstruction::new(
                1,
                &SystemInstruction::AdvanceNonceAccount,
                vec![2, 0, 1],
            )],
            Some(LoadedAddresses {
                writable: vec![nonce],
                readonly: vec![],
            }),
        );
        assert_eq!(SanitizedMessage::get_durable_nonce(&message), Some(&nonce));
        assert_eq!(
            SVMMessage::get_durable_nonce(&message, require_static_nonce_account),
            (!require_static_nonce_account).then_some(&nonce)
        );
    }
}

#[test]
fn test_get_ix_signers() {
    let signer0 = Pubkey::new_unique();
    let signer1 = Pubkey::new_unique();
    let non_signer = Pubkey::new_unique();
    let loader_key = Pubkey::new_unique();
    let instructions = vec![
        CompiledInstruction::new(3, &(), vec![2, 0]),
        CompiledInstruction::new(3, &(), vec![0, 1]),
        CompiledInstruction::new(3, &(), vec![0, 0]),
    ];

    let message = SanitizedMessage::try_from_legacy_message(
        legacy::Message::new_with_compiled_instructions(
            2,
            1,
            2,
            vec![signer0, signer1, non_signer, loader_key],
            Hash::default(),
            instructions,
        ),
        &HashSet::default(),
    )
    .unwrap();

    assert_eq!(
        SVMMessage::get_ix_signers(&message, 0).collect::<HashSet<_>>(),
        HashSet::from_iter([&signer0])
    );
    assert_eq!(
        SVMMessage::get_ix_signers(&message, 1).collect::<HashSet<_>>(),
        HashSet::from_iter([&signer0, &signer1])
    );
    assert_eq!(
        SVMMessage::get_ix_signers(&message, 2).collect::<HashSet<_>>(),
        HashSet::from_iter([&signer0])
    );
    assert_eq!(
        SVMMessage::get_ix_signers(&message, 3).collect::<HashSet<_>>(),
        HashSet::default()
    );
}
