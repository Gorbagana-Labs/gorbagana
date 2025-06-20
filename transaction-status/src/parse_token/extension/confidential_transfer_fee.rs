use {
    super::*,
    spl_token_2022::{
        extension::confidential_transfer_fee::instruction::*,
        instruction::{decode_instruction_data, decode_instruction_type},
        gorbagana_zk_sdk::encryption::pod::elgamal::PodElGamalPubkey,
    },
};

pub(in crate::parse_token) fn parse_confidential_transfer_fee_instruction(
    instruction_data: &[u8],
    account_indexes: &[u8],
    account_keys: &AccountKeys,
) -> Result<ParsedInstructionEnum, ParseInstructionError> {
    match decode_instruction_type(instruction_data)
        .map_err(|_| ParseInstructionError::InstructionNotParsable(ParsableProgram::SplToken))?
    {
        ConfidentialTransferFeeInstruction::InitializeConfidentialTransferFeeConfig => {
            check_num_token_accounts(account_indexes, 1)?;
            let transfer_fee_config: InitializeConfidentialTransferFeeConfigData =
                *decode_instruction_data(instruction_data).map_err(|_| {
                    ParseInstructionError::InstructionNotParsable(ParsableProgram::SplToken)
                })?;
            let mut value = json!({
                "mint": account_keys[account_indexes[0] as usize].to_string(),
                "withdrawWithheldAuthorityElGamalPubkey": Option::<PodElGamalPubkey>::from(transfer_fee_config.withdraw_withheld_authority_elgamal_pubkey).map(|k| k.to_string()),
            });
            let map = value.as_object_mut().unwrap();
            if let Some(authority) = Option::<Pubkey>::from(transfer_fee_config.authority) {
                map.insert("authority".to_string(), json!(authority.to_string()));
            }
            Ok(ParsedInstructionEnum {
                instruction_type: "initializeConfidentialTransferFeeConfig".to_string(),
                info: value,
            })
        }
        ConfidentialTransferFeeInstruction::WithdrawWithheldTokensFromMint => {
            check_num_token_accounts(account_indexes, 4)?;
            let withdraw_withheld_data: WithdrawWithheldTokensFromMintData =
                *decode_instruction_data(instruction_data).map_err(|_| {
                    ParseInstructionError::InstructionNotParsable(ParsableProgram::SplToken)
                })?;
            let proof_instruction_offset: i8 = withdraw_withheld_data.proof_instruction_offset;
            let mut value = json!({
                "mint": account_keys[account_indexes[0] as usize].to_string(),
                "feeRecipient": account_keys[account_indexes[1] as usize].to_string(),
                "proofInstructionOffset": proof_instruction_offset,
                "newDecryptableAvailableBalance": format!("{}", withdraw_withheld_data.new_decryptable_available_balance),
            });
            let map = value.as_object_mut().unwrap();
            let offset = if proof_instruction_offset == 0 {
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
                "withdrawWithheldAuthority",
                "multisigWithdrawWithheldAuthority",
            );
            Ok(ParsedInstructionEnum {
                instruction_type: "withdrawWithheldConfidentialTransferTokensFromMint".to_string(),
                info: value,
            })
        }
        ConfidentialTransferFeeInstruction::WithdrawWithheldTokensFromAccounts => {
            let withdraw_withheld_data: WithdrawWithheldTokensFromAccountsData =
                *decode_instruction_data(instruction_data).map_err(|_| {
                    ParseInstructionError::InstructionNotParsable(ParsableProgram::SplToken)
                })?;
            let num_token_accounts = withdraw_withheld_data.num_token_accounts;
            check_num_token_accounts(account_indexes, 4 + num_token_accounts as usize)?;
            let proof_instruction_offset: i8 = withdraw_withheld_data.proof_instruction_offset;
            let mut value = json!({
                "mint": account_keys[account_indexes[0] as usize].to_string(),
                "feeRecipient": account_keys[account_indexes[1] as usize].to_string(),
                "proofInstructionOffset": proof_instruction_offset,
                "newDecryptableAvailableBalance": format!("{}", withdraw_withheld_data.new_decryptable_available_balance),
            });
            let map = value.as_object_mut().unwrap();
            let first_source_account_index = account_indexes
                .len()
                .saturating_sub(num_token_accounts as usize);
            let offset = if proof_instruction_offset == 0 {
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
                if first_source_account_index > 4 {
                    // Assume that the extra account is a proof account and not a multisig
                    // signer. This might be wrong, but it's the best possible option.
                    map.insert(
                        "proofAccount".to_string(),
                        json!(account_keys[account_indexes[3] as usize].to_string()),
                    );
                    4
                } else {
                    3
                }
            };
            let mut source_accounts: Vec<String> = vec![];
            for i in account_indexes[first_source_account_index..].iter() {
                source_accounts.push(account_keys[*i as usize].to_string());
            }
            map.insert("sourceAccounts".to_string(), json!(source_accounts));
            parse_signers(
                map,
                offset,
                account_keys,
                &account_indexes[..first_source_account_index],
                "withdrawWithheldAuthority",
                "multisigWithdrawWithheldAuthority",
            );
            Ok(ParsedInstructionEnum {
                instruction_type: "withdrawWithheldConfidentialTransferTokensFromAccounts"
                    .to_string(),
                info: value,
            })
        }
        ConfidentialTransferFeeInstruction::HarvestWithheldTokensToMint => {
            check_num_token_accounts(account_indexes, 1)?;
            let mut value = json!({
                "mint": account_keys[account_indexes[0] as usize].to_string(),

            });
            let map = value.as_object_mut().unwrap();
            let mut source_accounts: Vec<String> = vec![];
            for i in account_indexes.iter().skip(1) {
                source_accounts.push(account_keys[*i as usize].to_string());
            }
            map.insert("sourceAccounts".to_string(), json!(source_accounts));
            Ok(ParsedInstructionEnum {
                instruction_type: "harvestWithheldConfidentialTransferTokensToMint".to_string(),
                info: value,
            })
        }
        ConfidentialTransferFeeInstruction::EnableHarvestToMint => {
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
                instruction_type: "enableConfidentialTransferFeeHarvestToMint".to_string(),
                info: value,
            })
        }
        ConfidentialTransferFeeInstruction::DisableHarvestToMint => {
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
                instruction_type: "disableConfidentialTransferFeeHarvestToMint".to_string(),
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
            extension::confidential_transfer_fee::instruction::{
                inner_withdraw_withheld_tokens_from_accounts,
                inner_withdraw_withheld_tokens_from_mint,
            },
            gorbagana_zk_sdk::{
                encryption::pod::auth_encryption::PodAeCiphertext,
                zk_elgamal_proof_program::proof_data::CiphertextCiphertextEqualityProofData,
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
    fn test_withdraw_from_accounts() {
        for location in [
            ProofLocation::InstructionOffset(
                NonZero::new(1).unwrap(),
                ProofData::InstructionData(&CiphertextCiphertextEqualityProofData::zeroed()),
            ),
            ProofLocation::InstructionOffset(
                NonZero::new(1).unwrap(),
                ProofData::RecordAccount(&Pubkey::new_unique(), 0),
            ),
            ProofLocation::ContextStateAccount(&Pubkey::new_unique()),
        ] {
            let instruction = inner_withdraw_withheld_tokens_from_accounts(
                &spl_token_2022::id(),
                &Pubkey::new_unique(),
                &Pubkey::new_unique(),
                &PodAeCiphertext::default(),
                &Pubkey::new_unique(),
                &[],
                &[&Pubkey::new_unique(), &Pubkey::new_unique()],
                location,
            )
            .unwrap();
            check_no_panic(instruction);
        }
    }

    #[test]
    fn test_withdraw_from_mint() {
        for location in [
            ProofLocation::InstructionOffset(
                NonZero::new(1).unwrap(),
                ProofData::InstructionData(&CiphertextCiphertextEqualityProofData::zeroed()),
            ),
            ProofLocation::InstructionOffset(
                NonZero::new(1).unwrap(),
                ProofData::RecordAccount(&Pubkey::new_unique(), 0),
            ),
            ProofLocation::ContextStateAccount(&Pubkey::new_unique()),
        ] {
            let instruction = inner_withdraw_withheld_tokens_from_mint(
                &spl_token_2022::id(),
                &Pubkey::new_unique(),
                &Pubkey::new_unique(),
                &PodAeCiphertext::default(),
                &Pubkey::new_unique(),
                &[],
                location,
            )
            .unwrap();
            check_no_panic(instruction);
        }
    }
}
