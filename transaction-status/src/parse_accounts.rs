pub use gorbagana_transaction_status_client_types::{ParsedAccount, ParsedAccountSource};
use {
    agave_reserved_account_keys::ReservedAccountKeys,
    gorbagana_message::{v0::LoadedMessage, Message},
};

pub fn parse_legacy_message_accounts(message: &Message) -> Vec<ParsedAccount> {
    let reserved_account_keys = ReservedAccountKeys::new_all_activated().active;
    let mut accounts: Vec<ParsedAccount> = vec![];
    for (i, account_key) in message.account_keys.iter().enumerate() {
        accounts.push(ParsedAccount {
            pubkey: account_key.to_string(),
            writable: message.is_maybe_writable(i, Some(&reserved_account_keys)),
            signer: message.is_signer(i),
            source: Some(ParsedAccountSource::Transaction),
        });
    }
    accounts
}

pub fn parse_v0_message_accounts(message: &LoadedMessage) -> Vec<ParsedAccount> {
    let mut accounts: Vec<ParsedAccount> = vec![];
    for (i, account_key) in message.account_keys().iter().enumerate() {
        let source = if i < message.static_account_keys().len() {
            ParsedAccountSource::Transaction
        } else {
            ParsedAccountSource::LookupTable
        };
        accounts.push(ParsedAccount {
            pubkey: account_key.to_string(),
            writable: message.is_writable(i),
            signer: message.is_signer(i),
            source: Some(source),
        });
    }
    accounts
}

#[cfg(test)]
mod test {
    use {
        super::*,
        agave_reserved_account_keys::ReservedAccountKeys,
        gorbagana_message::{v0, v0::LoadedAddresses, MessageHeader},
        gorbagana_pubkey::Pubkey,
    };

    #[test]
    fn test_parse_legacy_message_accounts() {
        let pubkey0 = Pubkey::new_unique();
        let pubkey1 = Pubkey::new_unique();
        let pubkey2 = Pubkey::new_unique();
        let pubkey3 = Pubkey::new_unique();
        let message = Message {
            header: MessageHeader {
                num_required_signatures: 2,
                num_readonly_signed_accounts: 1,
                num_readonly_unsigned_accounts: 1,
            },
            account_keys: vec![pubkey0, pubkey1, pubkey2, pubkey3],
            ..Message::default()
        };

        assert_eq!(
            parse_legacy_message_accounts(&message),
            vec![
                ParsedAccount {
                    pubkey: pubkey0.to_string(),
                    writable: true,
                    signer: true,
                    source: Some(ParsedAccountSource::Transaction),
                },
                ParsedAccount {
                    pubkey: pubkey1.to_string(),
                    writable: false,
                    signer: true,
                    source: Some(ParsedAccountSource::Transaction),
                },
                ParsedAccount {
                    pubkey: pubkey2.to_string(),
                    writable: true,
                    signer: false,
                    source: Some(ParsedAccountSource::Transaction),
                },
                ParsedAccount {
                    pubkey: pubkey3.to_string(),
                    writable: false,
                    signer: false,
                    source: Some(ParsedAccountSource::Transaction),
                },
            ]
        );
    }

    #[test]
    fn test_parse_v0_message_accounts() {
        let pubkey0 = Pubkey::new_unique();
        let pubkey1 = Pubkey::new_unique();
        let pubkey2 = Pubkey::new_unique();
        let pubkey3 = Pubkey::new_unique();
        let pubkey4 = Pubkey::new_unique();
        let pubkey5 = Pubkey::new_unique();
        let message = LoadedMessage::new(
            v0::Message {
                header: MessageHeader {
                    num_required_signatures: 2,
                    num_readonly_signed_accounts: 1,
                    num_readonly_unsigned_accounts: 1,
                },
                account_keys: vec![pubkey0, pubkey1, pubkey2, pubkey3],
                ..v0::Message::default()
            },
            LoadedAddresses {
                writable: vec![pubkey4],
                readonly: vec![pubkey5],
            },
            &ReservedAccountKeys::empty_key_set(),
        );

        assert_eq!(
            parse_v0_message_accounts(&message),
            vec![
                ParsedAccount {
                    pubkey: pubkey0.to_string(),
                    writable: true,
                    signer: true,
                    source: Some(ParsedAccountSource::Transaction),
                },
                ParsedAccount {
                    pubkey: pubkey1.to_string(),
                    writable: false,
                    signer: true,
                    source: Some(ParsedAccountSource::Transaction),
                },
                ParsedAccount {
                    pubkey: pubkey2.to_string(),
                    writable: true,
                    signer: false,
                    source: Some(ParsedAccountSource::Transaction),
                },
                ParsedAccount {
                    pubkey: pubkey3.to_string(),
                    writable: false,
                    signer: false,
                    source: Some(ParsedAccountSource::Transaction),
                },
                ParsedAccount {
                    pubkey: pubkey4.to_string(),
                    writable: true,
                    signer: false,
                    source: Some(ParsedAccountSource::LookupTable),
                },
                ParsedAccount {
                    pubkey: pubkey5.to_string(),
                    writable: false,
                    signer: false,
                    source: Some(ParsedAccountSource::LookupTable),
                },
            ]
        );
    }
}
