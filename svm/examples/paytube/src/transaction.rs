//! PayTube's custom transaction format, tailored specifically for SOL or SPL
//! token transfers.
//!
//! Mostly for demonstration purposes, to show how projects may use completely
//! different transactions in their protocol, then convert the resulting state
//! transitions into the necessary transactions for the base chain - in this
//! case Gorbagana.

use {
    gorbagana_instruction::Instruction as GorbaganaInstruction,
    gorbagana_pubkey::Pubkey,
    gorbagana_system_interface::instruction as system_instruction,
    gorbagana_transaction::{
        sanitized::SanitizedTransaction as GorbaganaSanitizedTransaction,
        Transaction as GorbaganaTransaction,
    },
    spl_associated_token_account::get_associated_token_address,
    std::collections::HashSet,
};

/// A simple PayTube transaction. Transfers SPL tokens or SOL from one account
/// to another.
///
/// A `None` value for `mint` represents native SOL.
pub struct PayTubeTransaction {
    pub mint: Option<Pubkey>,
    pub from: Pubkey,
    pub to: Pubkey,
    pub amount: u64,
}

impl From<&PayTubeTransaction> for GorbaganaInstruction {
    fn from(value: &PayTubeTransaction) -> Self {
        let PayTubeTransaction {
            mint,
            from,
            to,
            amount,
        } = value;
        if let Some(mint) = mint {
            let source_pubkey = get_associated_token_address(from, mint);
            let destination_pubkey = get_associated_token_address(to, mint);
            return spl_token::instruction::transfer(
                &spl_token::id(),
                &source_pubkey,
                &destination_pubkey,
                from,
                &[],
                *amount,
            )
            .unwrap();
        }
        system_instruction::transfer(from, to, *amount)
    }
}

impl From<&PayTubeTransaction> for GorbaganaTransaction {
    fn from(value: &PayTubeTransaction) -> Self {
        GorbaganaTransaction::new_with_payer(&[GorbaganaInstruction::from(value)], Some(&value.from))
    }
}

impl From<&PayTubeTransaction> for GorbaganaSanitizedTransaction {
    fn from(value: &PayTubeTransaction) -> Self {
        GorbaganaSanitizedTransaction::try_from_legacy_transaction(
            GorbaganaTransaction::from(value),
            &HashSet::new(),
        )
        .unwrap()
    }
}

/// Create a batch of Gorbagana transactions, for the Gorbagana SVM's transaction
/// processor, from a batch of PayTube instructions.
pub fn create_svm_transactions(
    paytube_transactions: &[PayTubeTransaction],
) -> Vec<GorbaganaSanitizedTransaction> {
    paytube_transactions
        .iter()
        .map(GorbaganaSanitizedTransaction::from)
        .collect()
}
