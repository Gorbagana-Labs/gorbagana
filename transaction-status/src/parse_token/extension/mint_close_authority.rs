use {super::*, gorbagana_program_option::COption, gorbagana_pubkey::Pubkey};

pub(in crate::parse_token) fn parse_initialize_mint_close_authority_instruction(
    close_authority: COption<Pubkey>,
    account_indexes: &[u8],
    account_keys: &AccountKeys,
) -> Result<ParsedInstructionEnum, ParseInstructionError> {
    check_num_token_accounts(account_indexes, 1)?;
    Ok(ParsedInstructionEnum {
        instruction_type: "initializeMintCloseAuthority".to_string(),
        info: json!({
            "mint": account_keys[account_indexes[0] as usize].to_string(),
            "newAuthority": map_coption_pubkey(close_authority),
        }),
    })
}

#[cfg(test)]
mod test {
    use {
        super::*, serde_json::Value, gorbagana_message::Message, gorbagana_pubkey::Pubkey,
        spl_token_2022::instruction::*,
    };

    #[test]
    fn test_parse_initialize_mint_close_authority_instruction() {
        let mint_pubkey = Pubkey::new_unique();
        let close_authority = Pubkey::new_unique();
        let mint_close_authority_ix = initialize_mint_close_authority(
            &spl_token_2022::id(),
            &mint_pubkey,
            Some(&close_authority),
        )
        .unwrap();
        let message = Message::new(&[mint_close_authority_ix], None);
        let compiled_instruction = &message.instructions[0];
        assert_eq!(
            parse_token(
                compiled_instruction,
                &AccountKeys::new(&message.account_keys, None)
            )
            .unwrap(),
            ParsedInstructionEnum {
                instruction_type: "initializeMintCloseAuthority".to_string(),
                info: json!({
                    "mint": mint_pubkey.to_string(),
                    "newAuthority": close_authority.to_string(),
                })
            }
        );

        let mint_close_authority_ix =
            initialize_mint_close_authority(&spl_token_2022::id(), &mint_pubkey, None).unwrap();
        let message = Message::new(&[mint_close_authority_ix], None);
        let compiled_instruction = &message.instructions[0];
        assert_eq!(
            parse_token(
                compiled_instruction,
                &AccountKeys::new(&message.account_keys, None)
            )
            .unwrap(),
            ParsedInstructionEnum {
                instruction_type: "initializeMintCloseAuthority".to_string(),
                info: json!({
                    "mint": mint_pubkey.to_string(),
                    "newAuthority": Value::Null,
                })
            }
        );
    }
}
