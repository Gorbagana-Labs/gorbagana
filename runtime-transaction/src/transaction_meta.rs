//! Transaction Meta contains data that follows a transaction through the
//! execution pipeline in runtime. Examples of metadata could be limits
//! specified by compute-budget instructions, simple-vote flag, transaction
//! costs, durable nonce account etc;
//!
//! The premise is if anything qualifies as metadata, then it must be valid
//! and available as long as the transaction itself is valid and available.
//! Hence they are not Option<T> type. Their visibility at different states
//! are defined in traits.
//!
//! The StaticMeta and DynamicMeta traits are accessor traits on the
//! RuntimeTransaction types, not the TransactionMeta itself.
//!
use {
    gorbagana_compute_budget_instruction::compute_budget_instruction_details::ComputeBudgetInstructionDetails,
    gorbagana_hash::Hash, gorbagana_message::TransactionSignatureDetails,
};

/// metadata can be extracted statically from sanitized transaction,
/// for example: message hash, simple-vote-tx flag, limits set by instructions
pub trait StaticMeta {
    fn message_hash(&self) -> &Hash;
    fn is_simple_vote_transaction(&self) -> bool;
    fn signature_details(&self) -> &TransactionSignatureDetails;
    fn compute_budget_instruction_details(&self) -> &ComputeBudgetInstructionDetails;
    fn instruction_data_len(&self) -> u16;
}

/// Statically loaded meta is a supertrait of Dynamically loaded meta, when
/// transaction transited successfully into dynamically loaded, it should
/// have both meta data populated and available.
/// Dynamic metadata available after accounts addresses are loaded from
/// on-chain ALT, examples are: transaction usage costs, nonce account.
pub trait DynamicMeta: StaticMeta {}

#[cfg_attr(feature = "dev-context-only-utils", derive(Clone))]
#[derive(Debug)]
pub struct TransactionMeta {
    pub(crate) message_hash: Hash,
    pub(crate) is_simple_vote_transaction: bool,
    pub(crate) signature_details: TransactionSignatureDetails,
    pub(crate) compute_budget_instruction_details: ComputeBudgetInstructionDetails,
    pub(crate) instruction_data_len: u16,
}
