#[cfg(feature = "dev-context-only-utils")]
use gorbagana_compute_budget_instruction::compute_budget_instruction_details::ComputeBudgetInstructionDetails;
use {
    crate::block_cost_limits, gorbagana_pubkey::Pubkey,
    gorbagana_runtime_transaction::transaction_meta::StaticMeta,
    gorbagana_svm_transaction::svm_message::SVMMessage,
};

/// `TransactionCost`` is used to represent resources required to process a
/// transaction, denominated in Compute Units (CUs). Resources required to
/// process a regular transaction often include an array of variables, such as
/// execution cost, loaded bytes, write lock and read lock etc.
///
/// SimpleVote has a simpler and pre-determined format. It has:
///  - 1 or 2 signatures
///  - 2 write locks
///  - 1 vote instruction
///  - less than 32k (page size) accounts to load
///
/// Its cost therefore can be static #33269.
const SIMPLE_VOTE_USAGE_COST: u64 = 3428;

#[derive(Debug)]
pub enum TransactionCost<'a, Tx> {
    SimpleVote { transaction: &'a Tx },
    Transaction(UsageCostDetails<'a, Tx>),
}

impl<Tx> TransactionCost<'_, Tx> {
    pub fn sum(&self) -> u64 {
        #![allow(clippy::assertions_on_constants)]
        match self {
            Self::SimpleVote { .. } => {
                const _: () = assert!(
                    SIMPLE_VOTE_USAGE_COST
                        == gorbagana_vote_program::vote_processor::DEFAULT_COMPUTE_UNITS
                            + block_cost_limits::SIGNATURE_COST
                            + 2 * block_cost_limits::WRITE_LOCK_UNITS
                            + 8
                );

                SIMPLE_VOTE_USAGE_COST
            }
            Self::Transaction(usage_cost) => usage_cost.sum(),
        }
    }

    pub fn programs_execution_cost(&self) -> u64 {
        match self {
            Self::SimpleVote { .. } => gorbagana_vote_program::vote_processor::DEFAULT_COMPUTE_UNITS,
            Self::Transaction(usage_cost) => usage_cost.programs_execution_cost,
        }
    }

    pub fn is_simple_vote(&self) -> bool {
        match self {
            Self::SimpleVote { .. } => true,
            Self::Transaction(_) => false,
        }
    }

    pub fn data_bytes_cost(&self) -> u16 {
        match self {
            Self::SimpleVote { .. } => 0,
            Self::Transaction(usage_cost) => usage_cost.data_bytes_cost,
        }
    }

    pub fn allocated_accounts_data_size(&self) -> u64 {
        match self {
            Self::SimpleVote { .. } => 0,
            Self::Transaction(usage_cost) => usage_cost.allocated_accounts_data_size,
        }
    }

    pub fn loaded_accounts_data_size_cost(&self) -> u64 {
        match self {
            Self::SimpleVote { .. } => 8, // simple-vote loads less than 32K account data,
            // the cost round up to be one page (32K) cost: 8CU
            Self::Transaction(usage_cost) => usage_cost.loaded_accounts_data_size_cost,
        }
    }

    pub fn signature_cost(&self) -> u64 {
        match self {
            Self::SimpleVote { .. } => block_cost_limits::SIGNATURE_COST,
            Self::Transaction(usage_cost) => usage_cost.signature_cost,
        }
    }

    pub fn write_lock_cost(&self) -> u64 {
        match self {
            Self::SimpleVote { .. } => block_cost_limits::WRITE_LOCK_UNITS.saturating_mul(2),
            Self::Transaction(usage_cost) => usage_cost.write_lock_cost,
        }
    }
}

impl<Tx: SVMMessage> TransactionCost<'_, Tx> {
    pub fn writable_accounts(&self) -> impl Iterator<Item = &Pubkey> {
        let transaction = match self {
            Self::SimpleVote { transaction } => transaction,
            Self::Transaction(usage_cost) => usage_cost.transaction,
        };
        transaction
            .account_keys()
            .iter()
            .enumerate()
            .filter_map(|(index, key)| transaction.is_writable(index).then_some(key))
    }
}

impl<Tx: StaticMeta> TransactionCost<'_, Tx> {
    pub fn num_transaction_signatures(&self) -> u64 {
        match self {
            Self::SimpleVote { .. } => 1,
            Self::Transaction(usage_cost) => usage_cost
                .transaction
                .signature_details()
                .num_transaction_signatures(),
        }
    }

    pub fn num_secp256k1_instruction_signatures(&self) -> u64 {
        match self {
            Self::SimpleVote { .. } => 0,
            Self::Transaction(usage_cost) => usage_cost
                .transaction
                .signature_details()
                .num_secp256k1_instruction_signatures(),
        }
    }

    pub fn num_ed25519_instruction_signatures(&self) -> u64 {
        match self {
            Self::SimpleVote { .. } => 0,
            Self::Transaction(usage_cost) => usage_cost
                .transaction
                .signature_details()
                .num_ed25519_instruction_signatures(),
        }
    }

    pub fn num_secp256r1_instruction_signatures(&self) -> u64 {
        match self {
            Self::SimpleVote { .. } => 0,
            Self::Transaction(usage_cost) => usage_cost
                .transaction
                .signature_details()
                .num_secp256r1_instruction_signatures(),
        }
    }
}

// costs are stored in number of 'compute unit's
#[derive(Debug)]
pub struct UsageCostDetails<'a, Tx> {
    pub transaction: &'a Tx,
    pub signature_cost: u64,
    pub write_lock_cost: u64,
    pub data_bytes_cost: u16,
    pub programs_execution_cost: u64,
    pub loaded_accounts_data_size_cost: u64,
    pub allocated_accounts_data_size: u64,
}

impl<Tx> UsageCostDetails<'_, Tx> {
    pub fn sum(&self) -> u64 {
        self.signature_cost
            .saturating_add(self.write_lock_cost)
            .saturating_add(u64::from(self.data_bytes_cost))
            .saturating_add(self.programs_execution_cost)
            .saturating_add(self.loaded_accounts_data_size_cost)
    }
}

#[cfg(feature = "dev-context-only-utils")]
#[derive(Debug)]
pub struct WritableKeysTransaction(pub Vec<Pubkey>);

#[cfg(feature = "dev-context-only-utils")]
impl gorbagana_svm_transaction::svm_message::SVMMessage for WritableKeysTransaction {
    fn num_transaction_signatures(&self) -> u64 {
        unimplemented!("WritableKeysTransaction::num_transaction_signatures")
    }

    fn num_write_locks(&self) -> u64 {
        unimplemented!("WritableKeysTransaction::num_write_locks")
    }

    fn recent_blockhash(&self) -> &gorbagana_hash::Hash {
        unimplemented!("WritableKeysTransaction::recent_blockhash")
    }

    fn num_instructions(&self) -> usize {
        unimplemented!("WritableKeysTransaction::num_instructions")
    }

    fn instructions_iter(
        &self,
    ) -> impl Iterator<Item = gorbagana_svm_transaction::instruction::SVMInstruction> {
        core::iter::empty()
    }

    fn program_instructions_iter(
        &self,
    ) -> impl Iterator<Item = (&Pubkey, gorbagana_svm_transaction::instruction::SVMInstruction)> + Clone
    {
        core::iter::empty()
    }

    fn static_account_keys(&self) -> &[Pubkey] {
        &self.0
    }

    fn account_keys(&self) -> gorbagana_message::AccountKeys {
        gorbagana_message::AccountKeys::new(&self.0, None)
    }

    fn fee_payer(&self) -> &Pubkey {
        unimplemented!("WritableKeysTransaction::fee_payer")
    }

    fn is_writable(&self, _index: usize) -> bool {
        true
    }

    fn is_signer(&self, _index: usize) -> bool {
        unimplemented!("WritableKeysTransaction::is_signer")
    }

    fn is_invoked(&self, _key_index: usize) -> bool {
        unimplemented!("WritableKeysTransaction::is_invoked")
    }

    fn num_lookup_tables(&self) -> usize {
        unimplemented!("WritableKeysTransaction::num_lookup_tables")
    }

    fn message_address_table_lookups(
        &self,
    ) -> impl Iterator<
        Item = gorbagana_svm_transaction::message_address_table_lookup::SVMMessageAddressTableLookup,
    > {
        core::iter::empty()
    }
}

#[cfg(feature = "dev-context-only-utils")]
impl gorbagana_svm_transaction::svm_transaction::SVMTransaction for WritableKeysTransaction {
    fn signature(&self) -> &gorbagana_signature::Signature {
        unimplemented!("WritableKeysTransaction::signature")
    }

    fn signatures(&self) -> &[gorbagana_signature::Signature] {
        unimplemented!("WritableKeysTransaction::signatures")
    }
}

#[cfg(feature = "dev-context-only-utils")]
impl gorbagana_runtime_transaction::transaction_meta::StaticMeta for WritableKeysTransaction {
    fn message_hash(&self) -> &gorbagana_hash::Hash {
        unimplemented!("WritableKeysTransaction::message_hash")
    }

    fn is_simple_vote_transaction(&self) -> bool {
        unimplemented!("WritableKeysTransaction::is_simple_vote_transaction")
    }

    fn signature_details(&self) -> &gorbagana_message::TransactionSignatureDetails {
        const DUMMY: gorbagana_message::TransactionSignatureDetails =
            gorbagana_message::TransactionSignatureDetails::new(0, 0, 0, 0);
        &DUMMY
    }

    fn compute_budget_instruction_details(&self) -> &ComputeBudgetInstructionDetails {
        unimplemented!("WritableKeysTransaction::compute_budget_instruction_details")
    }

    fn instruction_data_len(&self) -> u16 {
        unimplemented!("WritableKeysTransaction::instruction_data_len")
    }
}

#[cfg(feature = "dev-context-only-utils")]
impl gorbagana_runtime_transaction::transaction_with_meta::TransactionWithMeta
    for WritableKeysTransaction
{
    #[allow(refining_impl_trait)]
    fn as_sanitized_transaction(
        &self,
    ) -> std::borrow::Cow<gorbagana_transaction::sanitized::SanitizedTransaction> {
        unimplemented!("WritableKeysTransaction::as_sanitized_transaction");
    }

    fn to_versioned_transaction(&self) -> gorbagana_transaction::versioned::VersionedTransaction {
        unimplemented!("WritableKeysTransaction::to_versioned_transaction")
    }
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        crate::cost_model::CostModel,
        agave_feature_set::FeatureSet,
        agave_reserved_account_keys::ReservedAccountKeys,
        gorbagana_hash::Hash,
        gorbagana_keypair::Keypair,
        gorbagana_message::SimpleAddressLoader,
        gorbagana_runtime_transaction::runtime_transaction::RuntimeTransaction,
        gorbagana_transaction::{sanitized::MessageHash, versioned::VersionedTransaction},
        gorbagana_vote::vote_transaction,
        gorbagana_vote_program::vote_state::TowerSync,
    };

    fn get_example_transaction() -> VersionedTransaction {
        let node_keypair = Keypair::new();
        let vote_keypair = Keypair::new();
        let auth_keypair = Keypair::new();
        let transaction = vote_transaction::new_tower_sync_transaction(
            TowerSync::default(),
            Hash::default(),
            &node_keypair,
            &vote_keypair,
            &auth_keypair,
            None,
        );

        VersionedTransaction::from(transaction)
    }

    #[test]
    fn test_vote_transaction_cost() {
        gorbagana_logger::setup();

        // Create a sanitized vote transaction.
        let vote_transaction = RuntimeTransaction::try_create(
            get_example_transaction(),
            MessageHash::Compute,
            Some(true),
            SimpleAddressLoader::Disabled,
            &ReservedAccountKeys::empty_key_set(),
        )
        .unwrap();

        // Verify actual cost matches expected.
        let vote_cost = CostModel::calculate_cost(&vote_transaction, &FeatureSet::all_enabled());
        assert_eq!(SIMPLE_VOTE_USAGE_COST, vote_cost.sum());
    }

    #[test]
    fn test_non_vote_transaction_cost() {
        gorbagana_logger::setup();

        // Create a sanitized non-vote transaction.
        let non_vote_transaction = RuntimeTransaction::try_create(
            get_example_transaction(),
            MessageHash::Compute,
            Some(false),
            SimpleAddressLoader::Disabled,
            &ReservedAccountKeys::empty_key_set(),
        )
        .unwrap();

        // Compute expected cost.
        let signature_cost = 1440;
        let write_lock_cost = 600;
        let data_bytes_cost = 19;
        let programs_execution_cost = 3000;
        let loaded_accounts_data_size_cost = 16384;
        let expected_non_vote_cost = signature_cost
            + write_lock_cost
            + data_bytes_cost
            + programs_execution_cost
            + loaded_accounts_data_size_cost;

        // Verify actual cost matches expected.
        let non_vote_cost =
            CostModel::calculate_cost(&non_vote_transaction, &FeatureSet::all_enabled());
        assert_eq!(expected_non_vote_cost, non_vote_cost.sum());
    }
}
