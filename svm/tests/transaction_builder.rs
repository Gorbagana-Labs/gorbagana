use {
    agave_reserved_account_keys::ReservedAccountKeys,
    gorbagana_hash::Hash,
    gorbagana_instruction::AccountMeta,
    gorbagana_message::{
        compiled_instruction::CompiledInstruction,
        v0::{self, LoadedAddresses, MessageAddressTableLookup},
        AddressLoader, AddressLoaderError, Message, MessageHeader, VersionedMessage,
    },
    gorbagana_pubkey::Pubkey,
    gorbagana_signature::Signature,
    gorbagana_transaction::{
        sanitized::SanitizedTransaction,
        versioned::{sanitized::SanitizedVersionedTransaction, VersionedTransaction},
    },
    gorbagana_transaction_error::TransactionError,
    std::collections::{HashMap, HashSet},
};

#[derive(Default)]
pub struct SanitizedTransactionBuilder {
    instructions: Vec<InnerInstruction>,
    num_required_signatures: u8,
    num_readonly_signed_accounts: u8,
    num_readonly_unsigned_accounts: u8,
    signed_readonly_accounts: Vec<(Pubkey, Signature)>,
    signed_mutable_accounts: Vec<(Pubkey, Signature)>,
    unsigned_readonly_accounts: Vec<Pubkey>,
    unsigned_mutable_accounts: Vec<Pubkey>,
}

#[derive(PartialEq, Eq, Hash, Clone)]
enum AccountType {
    Readonly,
    Writable,
    SignerReadonly,
    SignerWritable,
}

struct InnerInstruction {
    program_id: Pubkey,
    accounts: Vec<(Pubkey, AccountType)>,
    data: Vec<u8>,
}

#[derive(Clone)]
struct MockLoader {}

// This implementation is only necessary if one is using account table lookups.
impl AddressLoader for MockLoader {
    fn load_addresses(
        self,
        _lookups: &[MessageAddressTableLookup],
    ) -> Result<LoadedAddresses, AddressLoaderError> {
        Ok(LoadedAddresses {
            writable: vec![],
            readonly: vec![],
        })
    }
}

impl SanitizedTransactionBuilder {
    pub fn create_instruction(
        &mut self,
        program_id: Pubkey,
        // The fee payer and the program id shall not appear in the accounts vector
        accounts: Vec<AccountMeta>,
        signatures: HashMap<Pubkey, Signature>,
        data: Vec<u8>,
    ) {
        let mut instruction = InnerInstruction {
            program_id,
            accounts: Vec::new(),
            data,
        };

        for item in &accounts {
            let acc_type = match (item.is_signer, item.is_writable) {
                (true, true) => {
                    self.num_required_signatures = self.num_required_signatures.saturating_add(1);
                    self.signed_mutable_accounts
                        .push((item.pubkey, signatures[&item.pubkey]));
                    AccountType::SignerWritable
                }
                (true, false) => {
                    self.num_required_signatures = self.num_required_signatures.saturating_add(1);
                    self.num_readonly_signed_accounts =
                        self.num_readonly_signed_accounts.saturating_add(1);
                    self.signed_readonly_accounts
                        .push((item.pubkey, signatures[&item.pubkey]));
                    AccountType::SignerReadonly
                }
                (false, true) => {
                    self.unsigned_mutable_accounts.push(item.pubkey);
                    AccountType::Writable
                }
                (false, false) => {
                    self.num_readonly_unsigned_accounts =
                        self.num_readonly_unsigned_accounts.saturating_add(1);
                    self.unsigned_readonly_accounts.push(item.pubkey);
                    AccountType::Readonly
                }
            };
            instruction.accounts.push((item.pubkey, acc_type));
        }

        self.instructions.push(instruction);
    }

    pub fn build(
        &mut self,
        block_hash: Hash,
        fee_payer: (Pubkey, Signature),
        v0_message: bool,
        ignore_reserved_accounts: bool,
    ) -> Result<SanitizedTransaction, TransactionError> {
        let mut account_keys = Vec::with_capacity(
            self.signed_mutable_accounts
                .len()
                .saturating_add(self.signed_readonly_accounts.len())
                .saturating_add(self.unsigned_mutable_accounts.len())
                .saturating_add(self.unsigned_readonly_accounts.len())
                .saturating_add(1),
        );
        let header = MessageHeader {
            // The fee payer always requires a signature so +1
            num_required_signatures: self.num_required_signatures.saturating_add(1),
            num_readonly_signed_accounts: self.num_readonly_signed_accounts,
            // The program id is always a readonly unsigned account
            num_readonly_unsigned_accounts: self.num_readonly_unsigned_accounts.saturating_add(1),
        };

        let mut compiled_instructions = Vec::new();

        let mut signatures = Vec::with_capacity(
            self.signed_mutable_accounts
                .len()
                .saturating_add(self.signed_readonly_accounts.len())
                .saturating_add(1),
        );
        let mut positions: HashMap<(Pubkey, AccountType), usize> = HashMap::new();

        account_keys.push(fee_payer.0);
        signatures.push(fee_payer.1);

        let mut positions_lambda = |key: &Pubkey, ty: AccountType| {
            positions.insert((*key, ty), account_keys.len());
            account_keys.push(*key);
        };

        self.signed_mutable_accounts
            .iter()
            .for_each(|(key, signature)| {
                positions_lambda(key, AccountType::SignerWritable);
                signatures.push(*signature);
            });
        self.signed_readonly_accounts
            .iter()
            .for_each(|(key, signature)| {
                positions_lambda(key, AccountType::SignerReadonly);
                signatures.push(*signature);
            });
        self.unsigned_mutable_accounts
            .iter()
            .for_each(|key| positions_lambda(key, AccountType::Writable));
        self.unsigned_readonly_accounts
            .iter()
            .for_each(|key| positions_lambda(key, AccountType::Readonly));

        let instructions = self.clean_up();

        for item in instructions {
            let accounts = item
                .accounts
                .iter()
                .map(|key| positions[key] as u8)
                .collect::<Vec<u8>>();
            let instruction = CompiledInstruction {
                program_id_index: push_and_return_index(item.program_id, &mut account_keys),
                accounts,
                data: item.data,
            };

            compiled_instructions.push(instruction);
        }

        let message = if v0_message {
            let message = v0::Message {
                header,
                account_keys,
                recent_blockhash: block_hash,
                instructions: compiled_instructions,
                address_table_lookups: vec![],
            };

            VersionedMessage::V0(message)
        } else {
            let message = Message {
                header,
                account_keys,
                recent_blockhash: block_hash,
                instructions: compiled_instructions,
            };

            VersionedMessage::Legacy(message)
        };

        let transaction = VersionedTransaction {
            signatures,
            message,
        };

        let sanitized_versioned_transaction =
            SanitizedVersionedTransaction::try_new(transaction).unwrap();

        let loader = MockLoader {};

        let reserved_active = &ReservedAccountKeys::new_all_activated().active;
        let all_inactive = HashSet::new();
        SanitizedTransaction::try_new(
            sanitized_versioned_transaction,
            Hash::new_unique(),
            false,
            loader,
            if ignore_reserved_accounts {
                &all_inactive
            } else {
                reserved_active
            },
        )
    }

    fn clean_up(&mut self) -> Vec<InnerInstruction> {
        let instructions = std::mem::take(&mut self.instructions);
        self.num_required_signatures = 0;
        self.num_readonly_signed_accounts = 0;
        self.num_readonly_unsigned_accounts = 0;
        self.signed_mutable_accounts.clear();
        self.signed_readonly_accounts.clear();
        self.unsigned_mutable_accounts.clear();
        self.unsigned_readonly_accounts.clear();

        instructions
    }
}

fn push_and_return_index(value: Pubkey, vector: &mut Vec<Pubkey>) -> u8 {
    vector.push(value);
    vector.len().saturating_sub(1) as u8
}
