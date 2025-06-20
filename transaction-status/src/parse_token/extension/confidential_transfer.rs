use {
    super::*,
    spl_token_2022::{
        extension::confidential_transfer::instruction::*,
        instruction::{decode_instruction_data, decode_instruction_type},
        gorbagana_zk_sdk::encryption::pod::elgamal::PodElGamalPubkey,
    },
};

pub(in crate::parse_token) fn parse_confidential_transfer_instruction(
    instruction_data: &[u8],
    account_indexes: &[u8],
    account_keys: &AccountKeys,
) -> Result<ParsedInstructionEnum, ParseInstructionError> {
    match decode_instruction_type(instruction_data)
        .map_err(|_| ParseInstructionError::InstructionNotParsable(ParsableProgram::SplToken))?
    {
        ConfidentialTransferInstruction::InitializeMint => {
            check_num_token_accounts(account_indexes, 1)?;
            let initialize_mint_data: InitializeMintData =
                *decode_instruction_data(instruction_data).map_err(|_| {
                    ParseInstructionError::InstructionNotParsable(ParsableProgram::SplToken)
                })?;
            let mut value = json!({
                "mint": account_keys[account_indexes[0] as usize].to_string(),
                "autoApproveNewAccounts": bool::from(initialize_mint_data.auto_approve_new_accounts),
                "auditorElGamalPubkey": Option::<PodElGamalPubkey>::from(initialize_mint_data.auditor_elgamal_pubkey).map(|k| k.to_string()),
            });
            let map = value.as_object_mut().unwrap();
            if let Some(authority) = Option::<Pubkey>::from(initialize_mint_data.authority) {
                map.insert("authority".to_string(), json!(authority.to_string()));
            }
            Ok(ParsedInstructionEnum {
                instruction_type: "initializeConfidentialTransferMint".to_string(),
                info: value,
            })
        }
        ConfidentialTransferInstruction::UpdateMint => {
            check_num_token_accounts(account_indexes, 2)?;
            let update_mint_data: UpdateMintData = *decode_instruction_data(instruction_data)
                .map_err(|_| {
                    ParseInstructionError::InstructionNotParsable(ParsableProgram::SplToken)
                })?;
            let value = json!({
                "mint": account_keys[account_indexes[0] as usize].to_string(),
                "confidentialTransferMintAuthority": account_keys[account_indexes[1] as usize].to_string(),
                "autoApproveNewAccounts": bool::from(update_mint_data.auto_approve_new_accounts),
                "auditorElGamalPubkey": Option::<PodElGamalPubkey>::from(update_mint_data.auditor_elgamal_pubkey).map(|k| k.to_string()),
            });
            Ok(ParsedInstructionEnum {
                instruction_type: "updateConfidentialTransferMint".to_string(),
                info: value,
            })
        }
        ConfidentialTransferInstruction::ConfigureAccount => {
            check_num_token_accounts(account_indexes, 4)?;
            let configure_account_data: ConfigureAccountInstructionData =
                *decode_instruction_data(instruction_data).map_err(|_| {
                    ParseInstructionError::InstructionNotParsable(ParsableProgram::SplToken)
                })?;
            let maximum_pending_balance_credit_counter: u64 = configure_account_data
                .maximum_pending_balance_credit_counter
                .into();
            let mut value = json!({
                "account": account_keys[account_indexes[0] as usize].to_string(),
                "mint": account_keys[account_indexes[1] as usize].to_string(),
                "decryptableZeroBalance": format!("{}", configure_account_data.decryptable_zero_balance),
                "maximumPendingBalanceCreditCounter": maximum_pending_balance_credit_counter,
                "proofInstructionOffset": configure_account_data.proof_instruction_offset,

            });
            let map = value.as_object_mut().unwrap();
            let offset = if configure_account_data.proof_instruction_offset == 0 {
                map.insert(
                    "proofContextStateAccount".to_string(),
                    json!(account_keys[account_indexes[2] as usize].to_string()),
                );
                3
            } else {
                map.insert(
                    "instructionsSysvar".to_string(),
                    json!(account_keys[account_indexes[2] as usize].to_string()),
                );
                // Assume that the extra account is a proof account and not a multisig
                // signer. This might be wrong, but it's the best possible option.
                if account_indexes.len() > 4 {
                    map.insert(
                        "recordAccount".to_string(),
                        json!(account_keys[account_indexes[3] as usize].to_string()),
                    );
                    4
                } else {
                    3
                }
            };
            parse_signers(
                map,
                offset,
                account_keys,
                account_indexes,
                "owner",
                "multisigOwner",
            );
            Ok(ParsedInstructionEnum {
                instruction_type: "configureConfidentialTransferAccount".to_string(),
                info: value,
            })
        }
        ConfidentialTransferInstruction::ApproveAccount => {
            check_num_token_accounts(account_indexes, 3)?;
            Ok(ParsedInstructionEnum {
                instruction_type: "approveConfidentialTransferAccount".to_string(),
                info: json!({
                    "account": account_keys[account_indexes[0] as usize].to_string(),
                    "mint": account_keys[account_indexes[1] as usize].to_string(),
                    "confidentialTransferAuditorAuthority": account_keys[account_indexes[2] as usize].to_string(),
                }),
            })
        }
        ConfidentialTransferInstruction::EmptyAccount => {
            check_num_token_accounts(account_indexes, 3)?;
            let empty_account_data: EmptyAccountInstructionData =
                *decode_instruction_data(instruction_data).map_err(|_| {
                    ParseInstructionError::InstructionNotParsable(ParsableProgram::SplToken)
                })?;
            let proof_instruction_offset: i8 = empty_account_data.proof_instruction_offset;
            let mut value = json!({
                "account": account_keys[account_indexes[0] as usize].to_string(),
                "proofInstructionOffset": proof_instruction_offset,

            });
            let map = value.as_object_mut().unwrap();
            let offset = if proof_instruction_offset == 0 {
                map.insert(
                    "proofContextStateAccount".to_string(),
                    json!(account_keys[account_indexes[1] as usize].to_string()),
                );
                2
            } else {
                map.insert(
                    "instructionsSysvar".to_string(),
                    json!(account_keys[account_indexes[1] as usize].to_string()),
                );
                if account_indexes.len() > 3 {
                    // Assume that the extra account is a proof account and not a multisig
                    // signer. This might be wrong, but it's the best possible option.
                    map.insert(
                        "recordAccount".to_string(),
                        json!(account_keys[account_indexes[2] as usize].to_string()),
                    );
                    3
                } else {
                    2
                }
            };
            parse_signers(
                map,
                offset,
                account_keys,
                account_indexes,
                "owner",
                "multisigOwner",
            );
            Ok(ParsedInstructionEnum {
                instruction_type: "emptyConfidentialTransferAccount".to_string(),
                info: value,
            })
        }
        ConfidentialTransferInstruction::Deposit => {
            check_num_token_accounts(account_indexes, 4)?;
            let deposit_data: DepositInstructionData = *decode_instruction_data(instruction_data)
                .map_err(|_| {
                ParseInstructionError::InstructionNotParsable(ParsableProgram::SplToken)
            })?;
            let amount: u64 = deposit_data.amount.into();
            let mut value = json!({
                "source": account_keys[account_indexes[0] as usize].to_string(),
                "destination": account_keys[account_indexes[1] as usize].to_string(),
                "mint": account_keys[account_indexes[2] as usize].to_string(),
                "amount": amount,
                "decimals": deposit_data.decimals,

            });
            let map = value.as_object_mut().unwrap();
            parse_signers(
                map,
                3,
                account_keys,
                account_indexes,
                "owner",
                "multisigOwner",
            );
            Ok(ParsedInstructionEnum {
                instruction_type: "depositConfidentialTransfer".to_string(),
                info: value,
            })
        }
        ConfidentialTransferInstruction::Withdraw => {
            check_num_token_accounts(account_indexes, 5)?;
            let withdrawal_data: WithdrawInstructionData =
                *decode_instruction_data(instruction_data).map_err(|_| {
                    ParseInstructionError::InstructionNotParsable(ParsableProgram::SplToken)
                })?;
            let amount: u64 = withdrawal_data.amount.into();
            let mut value = json!({
                "source": account_keys[account_indexes[0] as usize].to_string(),
                "destination": account_keys[account_indexes[1] as usize].to_string(),
                "mint": account_keys[account_indexes[2] as usize].to_string(),
                "amount": amount,
                "decimals": withdrawal_data.decimals,
                "newDecryptableAvailableBalance": format!("{}", withdrawal_data.new_decryptable_available_balance),
                "equalityProofInstructionOffset": withdrawal_data.equality_proof_instruction_offset,
                "rangeProofInstructionOffset": withdrawal_data.range_proof_instruction_offset,

            });
            let mut offset = 3;
            let map = value.as_object_mut().unwrap();
            if offset < account_indexes.len() - 1
                && (withdrawal_data.equality_proof_instruction_offset != 0
                    || withdrawal_data.range_proof_instruction_offset != 0)
            {
                map.insert(
                    "instructionsSysvar".to_string(),
                    json!(account_keys[account_indexes[offset] as usize].to_string()),
                );
                offset += 1;
            }

            // Assume that extra accounts are proof accounts and not multisig
            // signers. This might be wrong, but it's the best possible option.
            if offset < account_indexes.len() - 1 {
                let label = if withdrawal_data.equality_proof_instruction_offset == 0 {
                    "equalityProofContextStateAccount"
                } else {
                    "equalityProofRecordAccount"
                };
                map.insert(
                    label.to_string(),
                    json!(account_keys[account_indexes[offset] as usize].to_string()),
                );
                offset += 1;
            }

            if offset < account_indexes.len() - 1 {
                let label = if withdrawal_data.range_proof_instruction_offset == 0 {
                    "rangeProofContextStateAccount"
                } else {
                    "rangeProofRecordAccount"
                };
                map.insert(
                    label.to_string(),
                    json!(account_keys[account_indexes[offset] as usize].to_string()),
                );
                offset += 1;
            }
            parse_signers(
                map,
                offset,
                account_keys,
                account_indexes,
                "owner",
                "multisigOwner",
            );
            Ok(ParsedInstructionEnum {
                instruction_type: "withdrawConfidentialTransfer".to_string(),
                info: value,
            })
        }
        ConfidentialTransferInstruction::Transfer => {
            check_num_token_accounts(account_indexes, 4)?;
            let transfer_data: TransferInstructionData = *decode_instruction_data(instruction_data)
                .map_err(|_| {
                    ParseInstructionError::InstructionNotParsable(ParsableProgram::SplToken)
                })?;
            let mut value = json!({
                "source": account_keys[account_indexes[0] as usize].to_string(),
                "mint": account_keys[account_indexes[1] as usize].to_string(),
                "destination": account_keys[account_indexes[2] as usize].to_string(),
                "newSourceDecryptableAvailableBalance": format!("{}", transfer_data.new_source_decryptable_available_balance),
                "equalityProofInstructionOffset": transfer_data.equality_proof_instruction_offset,
                "ciphertextValidityProofInstructionOffset": transfer_data.ciphertext_validity_proof_instruction_offset,
                "rangeProofInstructionOffset": transfer_data.range_proof_instruction_offset,

            });
            let mut offset = 3;
            let map = value.as_object_mut().unwrap();
            if offset < account_indexes.len() - 1
                && (transfer_data.equality_proof_instruction_offset != 0
                    || transfer_data.ciphertext_validity_proof_instruction_offset != 0
                    || transfer_data.range_proof_instruction_offset != 0)
            {
                map.insert(
                    "instructionsSysvar".to_string(),
                    json!(account_keys[account_indexes[offset] as usize].to_string()),
                );
                offset += 1;
            }

            // Assume that extra accounts are proof accounts and not multisig
            // signers. This might be wrong, but it's the best possible option.
            if offset < account_indexes.len() - 1 {
                let label = if transfer_data.equality_proof_instruction_offset == 0 {
                    "equalityProofContextStateAccount"
                } else {
                    "equalityProofRecordAccount"
                };
                map.insert(
                    label.to_string(),
                    json!(account_keys[account_indexes[offset] as usize].to_string()),
                );
                offset += 1;
            }

            if offset < account_indexes.len() - 1 {
                let label = if transfer_data.ciphertext_validity_proof_instruction_offset == 0 {
                    "ciphertextValidityProofContextStateAccount"
                } else {
                    "ciphertextValidityProofRecordAccount"
                };
                map.insert(
                    label.to_string(),
                    json!(account_keys[account_indexes[offset] as usize].to_string()),
                );
                offset += 1;
            }

            if offset < account_indexes.len() - 1 {
                let label = if transfer_data.range_proof_instruction_offset == 0 {
                    "rangeProofContextStateAccount"
                } else {
                    "rangeProofRecordAccount"
                };
                map.insert(
                    label.to_string(),
                    json!(account_keys[account_indexes[offset] as usize].to_string()),
                );
                offset += 1;
            }

            parse_signers(
                map,
                offset,
                account_keys,
                account_indexes,
                "owner",
                "multisigOwner",
            );
            Ok(ParsedInstructionEnum {
                instruction_type: "confidentialTransfer".to_string(),
                info: value,
            })
        }
        ConfidentialTransferInstruction::TransferWithFee => {
            check_num_token_accounts(account_indexes, 4)?;
            let transfer_data: TransferWithFeeInstructionData =
                *decode_instruction_data(instruction_data).map_err(|_| {
                    ParseInstructionError::InstructionNotParsable(ParsableProgram::SplToken)
                })?;
            let equality_proof_instruction_offset: i8 =
                transfer_data.equality_proof_instruction_offset;
            let transfer_amount_ciphertext_validity_proof_instruction_offset: i8 =
                transfer_data.transfer_amount_ciphertext_validity_proof_instruction_offset;
            let fee_sigma_proof_instruction_offset: i8 =
                transfer_data.fee_sigma_proof_instruction_offset;
            let fee_ciphertext_validity_proof_instruction_offset: i8 =
                transfer_data.fee_ciphertext_validity_proof_instruction_offset;
            let range_proof_instruction_offset: i8 = transfer_data.range_proof_instruction_offset;
            let mut value = json!({
                "source": account_keys[account_indexes[0] as usize].to_string(),
                "mint": account_keys[account_indexes[1] as usize].to_string(),
                "destination": account_keys[account_indexes[2] as usize].to_string(),
                "newSourceDecryptableAvailableBalance": format!("{}", transfer_data.new_source_decryptable_available_balance),
                "equalityProofInstructionOffset": equality_proof_instruction_offset,
                "transferAmountCiphertextValidityProofInstructionOffset": transfer_amount_ciphertext_validity_proof_instruction_offset,
                "feeCiphertextValidityProofInstructionOffset": fee_ciphertext_validity_proof_instruction_offset,
                "feeSigmaProofInstructionOffset": fee_sigma_proof_instruction_offset,
                "rangeProofInstructionOffset": range_proof_instruction_offset,
            });

            let mut offset = 3;
            let map = value.as_object_mut().unwrap();
            if offset < account_indexes.len() - 1
                && (equality_proof_instruction_offset != 0
                    || transfer_amount_ciphertext_validity_proof_instruction_offset != 0
                    || fee_ciphertext_validity_proof_instruction_offset != 0
                    || fee_sigma_proof_instruction_offset != 0
                    || range_proof_instruction_offset != 0)
            {
                map.insert(
                    "instructionsSysvar".to_string(),
                    json!(account_keys[account_indexes[offset] as usize].to_string()),
                );
                offset += 1;
            }

            // Assume that extra accounts are proof accounts and not multisig
            // signers. This might be wrong, but it's the best possible option.
            if offset < account_indexes.len() - 1 {
                let label = if equality_proof_instruction_offset == 0 {
                    "equalityProofContextStateAccount"
                } else {
                    "equalityProofRecordAccount"
                };
                map.insert(
                    label.to_string(),
                    json!(account_keys[account_indexes[offset] as usize].to_string()),
                );
                offset += 1;
            }
            if offset < account_indexes.len() - 1 {
                let label = if transfer_amount_ciphertext_validity_proof_instruction_offset == 0 {
                    "transferAmountCiphertextValidityProofContextStateAccount"
                } else {
                    "transferAmountCiphertextValidityProofRecordAccount"
                };
                map.insert(
                    label.to_string(),
                    json!(account_keys[account_indexes[offset] as usize].to_string()),
                );
                offset += 1;
            }
            if offset < account_indexes.len() - 1 {
                let label = if fee_ciphertext_validity_proof_instruction_offset == 0 {
                    "feeCiphertextValidityProofContextStateAccount"
                } else {
                    "feeCiphertextValidityProofRecordAccount"
                };
                map.insert(
                    label.to_string(),
                    json!(account_keys[account_indexes[offset] as usize].to_string()),
                );
                offset += 1;
            }
            if offset < account_indexes.len() - 1 {
                let label = if fee_sigma_proof_instruction_offset == 0 {
                    "feeSigmaProofContextStateAccount"
                } else {
                    "feeSigmaProofRecordAccount"
                };
                map.insert(
                    label.to_string(),
                    json!(account_keys[account_indexes[offset] as usize].to_string()),
                );
                offset += 1;
            }
            if offset < account_indexes.len() - 1 {
                let label = if range_proof_instruction_offset == 0 {
                    "rangeProofContextStateAccount"
                } else {
                    "rangeProofRecordAccount"
                };
                map.insert(
                    label.to_string(),
                    json!(account_keys[account_indexes[offset] as usize].to_string()),
                );
                offset += 1;
            }
            parse_signers(
                map,
                offset,
                account_keys,
                account_indexes,
                "owner",
                "multisigOwner",
            );
            Ok(ParsedInstructionEnum {
                instruction_type: "confidentialTransferWithFee".to_string(),
                info: value,
            })
        }
        ConfidentialTransferInstruction::ApplyPendingBalance => {
            check_num_token_accounts(account_indexes, 2)?;
            let apply_pending_balance_data: ApplyPendingBalanceData =
                *decode_instruction_data(instruction_data).map_err(|_| {
                    ParseInstructionError::InstructionNotParsable(ParsableProgram::SplToken)
                })?;
            let expected_pending_balance_credit_counter: u64 = apply_pending_balance_data
                .expected_pending_balance_credit_counter
                .into();
            let mut value = json!({
                "account": account_keys[account_indexes[0] as usize].to_string(),
                "newDecryptableAvailableBalance": format!("{}", apply_pending_balance_data.new_decryptable_available_balance),
                "expectedPendingBalanceCreditCounter": expected_pending_balance_credit_counter,

            });
            let map = value.as_object_mut().unwrap();
            parse_signers(
                map,
                1,
                account_keys,
                account_indexes,
                "owner",
                "multisigOwner",
            );
            Ok(ParsedInstructionEnum {
                instruction_type: "applyPendingConfidentialTransferBalance".to_string(),
                info: value,
            })
        }
        ConfidentialTransferInstruction::EnableConfidentialCredits => {
            check_num_token_accounts(account_indexes, 2)?;
            let mut value = json!({
                "account": account_keys[account_indexes[0] as usize].to_string(),

            });
            let map = value.as_object_mut().unwrap();
            parse_signers(
                map,
                1,
                account_keys,
                account_indexes,
                "owner",
                "multisigOwner",
            );
            Ok(ParsedInstructionEnum {
                instruction_type: "enableConfidentialTransferConfidentialCredits".to_string(),
                info: value,
            })
        }
        ConfidentialTransferInstruction::DisableConfidentialCredits => {
            check_num_token_accounts(account_indexes, 2)?;
            let mut value = json!({
                "account": account_keys[account_indexes[0] as usize].to_string(),

            });
            let map = value.as_object_mut().unwrap();
            parse_signers(
                map,
                1,
                account_keys,
                account_indexes,
                "owner",
                "multisigOwner",
            );
            Ok(ParsedInstructionEnum {
                instruction_type: "disableConfidentialTransferConfidentialCredits".to_string(),
                info: value,
            })
        }
        ConfidentialTransferInstruction::EnableNonConfidentialCredits => {
            check_num_token_accounts(account_indexes, 2)?;
            let mut value = json!({
                "account": account_keys[account_indexes[0] as usize].to_string(),

            });
            let map = value.as_object_mut().unwrap();
            parse_signers(
                map,
                1,
                account_keys,
                account_indexes,
                "owner",
                "multisigOwner",
            );
            Ok(ParsedInstructionEnum {
                instruction_type: "enableConfidentialTransferNonConfidentialCredits".to_string(),
                info: value,
            })
        }
        ConfidentialTransferInstruction::DisableNonConfidentialCredits => {
            check_num_token_accounts(account_indexes, 2)?;
            let mut value = json!({
                "account": account_keys[account_indexes[0] as usize].to_string(),

            });
            let map = value.as_object_mut().unwrap();
            parse_signers(
                map,
                1,
                account_keys,
                account_indexes,
                "owner",
                "multisigOwner",
            );
            Ok(ParsedInstructionEnum {
                instruction_type: "disableConfidentialTransferNonConfidentialCredits".to_string(),
                info: value,
            })
        }
        ConfidentialTransferInstruction::ConfigureAccountWithRegistry => {
            check_num_token_accounts(account_indexes, 3)?;
            let value = json!({
                "account": account_keys[account_indexes[0] as usize].to_string(),
                "mint": account_keys[account_indexes[1] as usize].to_string(),
                "registry": account_keys[account_indexes[2] as usize].to_string(),
            });
            Ok(ParsedInstructionEnum {
                instruction_type: "configureConfidentialAccountWithRegistry".to_string(),
                info: value,
            })
        }
    }
}

#[cfg(test)]
mod test {
    use {
        super::*,
        bytemuck::Zeroable,
        gorbagana_instruction::{AccountMeta, Instruction},
        gorbagana_message::Message,
        gorbagana_pubkey::Pubkey,
        spl_token_2022::{
            extension::confidential_transfer::instruction::{
                initialize_mint, inner_configure_account, inner_empty_account, update_mint,
            },
            gorbagana_zk_sdk::{
                encryption::pod::{
                    auth_encryption::PodAeCiphertext, elgamal::PodElGamalCiphertext,
                },
                zk_elgamal_proof_program::proof_data::{
                    BatchedGroupedCiphertext3HandlesValidityProofData, BatchedRangeProofU128Data,
                    CiphertextCommitmentEqualityProofData, ZeroCiphertextProofData,
                },
            },
        },
        spl_token_confidential_transfer_proof_extraction::instruction::{ProofData, ProofLocation},
        std::num::NonZero,
    };

    fn check_no_panic(mut instruction: Instruction) {
        let account_meta = AccountMeta::new_readonly(Pubkey::new_unique(), false);
        for i in 0..20 {
            instruction.accounts = vec![account_meta.clone(); i];
            let message = Message::new(&[instruction.clone()], None);
            let compiled_instruction = &message.instructions[0];
            let _ = parse_token(
                compiled_instruction,
                &AccountKeys::new(&message.account_keys, None),
            );
        }
    }

    #[test]
    fn test_initialize() {
        let instruction = initialize_mint(
            &spl_token_2022::id(),
            &Pubkey::new_unique(),
            Some(Pubkey::new_unique()),
            true,
            None,
        )
        .unwrap();
        check_no_panic(instruction);
    }

    #[test]
    fn test_approve() {
        let instruction = approve_account(
            &spl_token_2022::id(),
            &Pubkey::new_unique(),
            &Pubkey::new_unique(),
            &Pubkey::new_unique(),
            &[],
        )
        .unwrap();
        check_no_panic(instruction);
    }

    #[test]
    fn test_update() {
        let instruction = update_mint(
            &spl_token_2022::id(),
            &Pubkey::new_unique(),
            &Pubkey::new_unique(),
            &[],
            true,
            None,
        )
        .unwrap();
        check_no_panic(instruction);
    }

    #[test]
    fn test_configure() {
        for location in [
            ProofLocation::InstructionOffset(
                NonZero::new(1).unwrap(),
                ProofData::InstructionData(&PubkeyValidityProofData::zeroed()),
            ),
            ProofLocation::InstructionOffset(
                NonZero::new(1).unwrap(),
                ProofData::RecordAccount(&Pubkey::new_unique(), 0),
            ),
            ProofLocation::ContextStateAccount(&Pubkey::new_unique()),
        ] {
            let instruction = inner_configure_account(
                &spl_token_2022::id(),
                &Pubkey::new_unique(),
                &Pubkey::new_unique(),
                &PodAeCiphertext::default(),
                10_000,
                &Pubkey::new_unique(),
                &[],
                location,
            )
            .unwrap();
            check_no_panic(instruction);
        }
    }

    #[test]
    fn test_empty_account() {
        for location in [
            ProofLocation::InstructionOffset(
                NonZero::new(1).unwrap(),
                ProofData::InstructionData(&ZeroCiphertextProofData::zeroed()),
            ),
            ProofLocation::InstructionOffset(
                NonZero::new(1).unwrap(),
                ProofData::RecordAccount(&Pubkey::new_unique(), 0),
            ),
            ProofLocation::ContextStateAccount(&Pubkey::new_unique()),
        ] {
            let instruction = inner_empty_account(
                &spl_token_2022::id(),
                &Pubkey::new_unique(),
                &Pubkey::new_unique(),
                &[],
                location,
            )
            .unwrap();
            check_no_panic(instruction);
        }
    }

    #[test]
    fn test_withdraw() {
        for (equality_proof_location, range_proof_location) in [
            (
                ProofLocation::InstructionOffset(
                    NonZero::new(1).unwrap(),
                    ProofData::InstructionData(&CiphertextCommitmentEqualityProofData::zeroed()),
                ),
                ProofLocation::InstructionOffset(
                    NonZero::new(3).unwrap(),
                    ProofData::InstructionData(&BatchedRangeProofU64Data::zeroed()),
                ),
            ),
            (
                ProofLocation::InstructionOffset(
                    NonZero::new(1).unwrap(),
                    ProofData::RecordAccount(&Pubkey::new_unique(), 0),
                ),
                ProofLocation::InstructionOffset(
                    NonZero::new(2).unwrap(),
                    ProofData::RecordAccount(&Pubkey::new_unique(), 0),
                ),
            ),
            (
                ProofLocation::ContextStateAccount(&Pubkey::new_unique()),
                ProofLocation::ContextStateAccount(&Pubkey::new_unique()),
            ),
        ] {
            let instruction = inner_withdraw(
                &spl_token_2022::id(),
                &Pubkey::new_unique(),
                &Pubkey::new_unique(),
                1,
                2,
                &PodAeCiphertext::default(),
                &Pubkey::new_unique(),
                &[],
                equality_proof_location,
                range_proof_location,
            )
            .unwrap();
            check_no_panic(instruction);
        }
    }

    #[test]
    fn test_transfer() {
        for (equality_proof_location, ciphertext_validity_proof_location, range_proof_location) in [
            (
                ProofLocation::InstructionOffset(
                    NonZero::new(1).unwrap(),
                    ProofData::InstructionData(&CiphertextCommitmentEqualityProofData::zeroed()),
                ),
                ProofLocation::InstructionOffset(
                    NonZero::new(2).unwrap(),
                    ProofData::InstructionData(
                        &BatchedGroupedCiphertext3HandlesValidityProofData::zeroed(),
                    ),
                ),
                ProofLocation::InstructionOffset(
                    NonZero::new(3).unwrap(),
                    ProofData::InstructionData(&BatchedRangeProofU128Data::zeroed()),
                ),
            ),
            (
                ProofLocation::InstructionOffset(
                    NonZero::new(1).unwrap(),
                    ProofData::RecordAccount(&Pubkey::new_unique(), 0),
                ),
                ProofLocation::InstructionOffset(
                    NonZero::new(2).unwrap(),
                    ProofData::RecordAccount(&Pubkey::new_unique(), 0),
                ),
                ProofLocation::InstructionOffset(
                    NonZero::new(3).unwrap(),
                    ProofData::RecordAccount(&Pubkey::new_unique(), 0),
                ),
            ),
            (
                ProofLocation::ContextStateAccount(&Pubkey::new_unique()),
                ProofLocation::ContextStateAccount(&Pubkey::new_unique()),
                ProofLocation::ContextStateAccount(&Pubkey::new_unique()),
            ),
        ] {
            let instruction = inner_transfer(
                &spl_token_2022::id(),
                &Pubkey::new_unique(),
                &Pubkey::new_unique(),
                &Pubkey::new_unique(),
                &PodAeCiphertext::default(),
                &PodElGamalCiphertext::default(),
                &PodElGamalCiphertext::default(),
                &Pubkey::new_unique(),
                &[],
                equality_proof_location,
                ciphertext_validity_proof_location,
                range_proof_location,
            )
            .unwrap();
            check_no_panic(instruction);
        }
    }

    #[test]
    fn test_transfer_with_fee() {
        for (
            equality_proof_location,
            transfer_amount_ciphertext_validity_proof_location,
            fee_sigma_proof_location,
            fee_ciphertext_validity_proof_location,
            range_proof_location,
        ) in [
            (
                ProofLocation::InstructionOffset(
                    NonZero::new(1).unwrap(),
                    ProofData::InstructionData(&CiphertextCommitmentEqualityProofData::zeroed()),
                ),
                ProofLocation::InstructionOffset(
                    NonZero::new(2).unwrap(),
                    ProofData::InstructionData(
                        &BatchedGroupedCiphertext3HandlesValidityProofData::zeroed(),
                    ),
                ),
                ProofLocation::InstructionOffset(
                    NonZero::new(3).unwrap(),
                    ProofData::InstructionData(&PercentageWithCapProofData::zeroed()),
                ),
                ProofLocation::InstructionOffset(
                    NonZero::new(4).unwrap(),
                    ProofData::InstructionData(
                        &BatchedGroupedCiphertext2HandlesValidityProofData::zeroed(),
                    ),
                ),
                ProofLocation::InstructionOffset(
                    NonZero::new(5).unwrap(),
                    ProofData::InstructionData(&BatchedRangeProofU256Data::zeroed()),
                ),
            ),
            (
                ProofLocation::InstructionOffset(
                    NonZero::new(1).unwrap(),
                    ProofData::RecordAccount(&Pubkey::new_unique(), 0),
                ),
                ProofLocation::InstructionOffset(
                    NonZero::new(2).unwrap(),
                    ProofData::RecordAccount(&Pubkey::new_unique(), 0),
                ),
                ProofLocation::InstructionOffset(
                    NonZero::new(3).unwrap(),
                    ProofData::RecordAccount(&Pubkey::new_unique(), 0),
                ),
                ProofLocation::InstructionOffset(
                    NonZero::new(4).unwrap(),
                    ProofData::RecordAccount(&Pubkey::new_unique(), 0),
                ),
                ProofLocation::InstructionOffset(
                    NonZero::new(5).unwrap(),
                    ProofData::RecordAccount(&Pubkey::new_unique(), 0),
                ),
            ),
            (
                ProofLocation::ContextStateAccount(&Pubkey::new_unique()),
                ProofLocation::ContextStateAccount(&Pubkey::new_unique()),
                ProofLocation::ContextStateAccount(&Pubkey::new_unique()),
                ProofLocation::ContextStateAccount(&Pubkey::new_unique()),
                ProofLocation::ContextStateAccount(&Pubkey::new_unique()),
            ),
        ] {
            let instruction = inner_transfer_with_fee(
                &spl_token_2022::id(),
                &Pubkey::new_unique(),
                &Pubkey::new_unique(),
                &Pubkey::new_unique(),
                &PodAeCiphertext::default(),
                &PodElGamalCiphertext::default(),
                &PodElGamalCiphertext::default(),
                &Pubkey::new_unique(),
                &[],
                equality_proof_location,
                transfer_amount_ciphertext_validity_proof_location,
                fee_sigma_proof_location,
                fee_ciphertext_validity_proof_location,
                range_proof_location,
            )
            .unwrap();
            check_no_panic(instruction);
        }
    }
}
