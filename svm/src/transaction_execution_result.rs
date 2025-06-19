// Re-exported since these have moved to `gorbagana_message`.
#[deprecated(
    since = "1.18.0",
    note = "Please use `gorbagana_message::inner_instruction` types instead"
)]
pub use gorbagana_message::inner_instruction::{InnerInstruction, InnerInstructionsList};
use {
    crate::account_loader::LoadedTransaction,
    gorbagana_program_runtime::loaded_programs::ProgramCacheEntry,
    gorbagana_pubkey::Pubkey,
    gorbagana_transaction_context::TransactionReturnData,
    gorbagana_transaction_error::TransactionResult,
    std::{collections::HashMap, sync::Arc},
};

#[derive(Debug, Default, Clone, PartialEq)]
pub struct TransactionLoadedAccountsStats {
    pub loaded_accounts_data_size: u32,
    pub loaded_accounts_count: usize,
}

#[derive(Debug, Clone)]
pub struct ExecutedTransaction {
    pub loaded_transaction: LoadedTransaction,
    pub execution_details: TransactionExecutionDetails,
    pub programs_modified_by_tx: HashMap<Pubkey, Arc<ProgramCacheEntry>>,
}

impl ExecutedTransaction {
    pub fn was_successful(&self) -> bool {
        self.execution_details.was_successful()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransactionExecutionDetails {
    pub status: TransactionResult<()>,
    pub log_messages: Option<Vec<String>>,
    pub inner_instructions: Option<InnerInstructionsList>,
    pub return_data: Option<TransactionReturnData>,
    pub executed_units: u64,
    /// The change in accounts data len for this transaction.
    /// NOTE: This value is valid IFF `status` is `Ok`.
    pub accounts_data_len_delta: i64,
}

impl TransactionExecutionDetails {
    pub fn was_successful(&self) -> bool {
        self.status.is_ok()
    }
}
