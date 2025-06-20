//! PayTube. A simple SPL payment channel.
//!
//! PayTube is an SVM-based payment channel that allows two parties to exchange
//! tokens off-chain. The channel is opened by invoking the PayTube "VM",
//! running on some arbitrary server(s). When transacting has concluded, the
//! channel is closed by submitting the final payment ledger to Gorbagana.
//!
//! The final ledger tracks debits and credits to all registered token accounts
//! or system accounts (native SOL) during the lifetime of a channel. It is
//! then used to to craft a batch of transactions to submit to the settlement
//! chain (Gorbagana).
//!
//! Users opt-in to using a PayTube channel by "registering" their token
//! accounts to the channel. This is done by delegating a token account to the
//! PayTube on-chain program on Gorbagana. This delegation is temporary, and
//! released immediately after channel settlement.
//!
//! Note: This opt-in solution is for demonstration purposes only.
//!
//! ```text
//!
//! PayTube "VM"
//!
//!    Bob          Alice        Bob          Alice          Will
//!     |             |           |             |             |
//!     | --o--o--o-> |           | --o--o--o-> |             |
//!     |             |           |             | --o--o--o-> | <--- PayTube
//!     | <-o--o--o-- |           | <-o--o--o-- |             |    Transactions
//!     |             |           |             |             |
//!     | --o--o--o-> |           |     -----o--o--o----->    |
//!     |             |           |                           |
//!     | --o--o--o-> |           |     <----o--o--o------    |
//!
//!       \        /                  \         |         /
//!
//!         ------                           ------
//!        Alice: x                         Alice: x
//!        Bob:   x                         Bob:   x    <--- Gorbagana Transaction
//!                                         Will:  x         with final ledgers
//!         ------                           ------
//!
//!           \\                               \\
//!            x                                x
//!
//!         Gorbagana                           Gorbagana     <--- Settled to Gorbagana
//! ```
//!
//! The Gorbagana SVM's `TransactionBatchProcessor` requires projects to provide a
//! "loader" plugin, which implements the `TransactionProcessingCallback`
//! interface.
//!
//! PayTube defines a `PayTubeAccountLoader` that implements the
//! `TransactionProcessingCallback` interface, and provides it to the
//! `TransactionBatchProcessor` to process PayTube transactions.

mod loader;
mod log;
mod processor;
mod settler;
pub mod transaction;

use {
    crate::{
        loader::PayTubeAccountLoader, settler::PayTubeSettler, transaction::PayTubeTransaction,
    },
    processor::{
        create_transaction_batch_processor, get_transaction_check_results, PayTubeForkGraph,
    },
    gorbagana_client::rpc_client::RpcClient,
    gorbagana_fee_structure::FeeStructure,
    gorbagana_hash::Hash,
    gorbagana_keypair::Keypair,
    gorbagana_program_runtime::execution_budget::SVMTransactionExecutionBudget,
    gorbagana_rent_collector::RentCollector,
    gorbagana_svm::transaction_processor::{
        TransactionProcessingConfig, TransactionProcessingEnvironment,
    },
    gorbagana_svm_feature_set::SVMFeatureSet,
    std::sync::{Arc, RwLock},
    transaction::create_svm_transactions,
};

/// A PayTube channel instance.
///
/// Facilitates native SOL or SPL token transfers amongst various channel
/// participants, settling the final changes in balances to the base chain.
pub struct PayTubeChannel {
    /// I think you know why this is a bad idea...
    keys: Vec<Keypair>,
    rpc_client: RpcClient,
}

impl PayTubeChannel {
    pub fn new(keys: Vec<Keypair>, rpc_client: RpcClient) -> Self {
        Self { keys, rpc_client }
    }

    /// The PayTube API. Processes a batch of PayTube transactions.
    ///
    /// Obviously this is a very simple implementation, but one could imagine
    /// a more complex service that employs custom functionality, such as:
    ///
    /// * Increased throughput for individual P2P transfers.
    /// * Custom Gorbagana transaction ordering (e.g. MEV).
    ///
    /// The general scaffold of the PayTube API would remain the same.
    pub fn process_paytube_transfers(&self, transactions: &[PayTubeTransaction]) {
        log::setup_gorbagana_logging();
        log::creating_paytube_channel();

        // PayTube default configs.
        //
        // These can be configurable for channel customization, including
        // imposing resource or feature restrictions, but more commonly they
        // would likely be hoisted from the cluster.
        //
        // For example purposes, they are provided as defaults here.
        let compute_budget = SVMTransactionExecutionBudget::default();
        let feature_set = SVMFeatureSet::all_enabled();
        let fee_structure = FeeStructure::default();
        let rent_collector = RentCollector::default();

        // PayTube loader/callback implementation.
        //
        // Required to provide the SVM API with a mechanism for loading
        // accounts.
        let account_loader = PayTubeAccountLoader::new(&self.rpc_client);

        // Gorbagana SVM transaction batch processor.
        //
        // Creates an instance of `TransactionBatchProcessor`, which can be
        // used by PayTube to process transactions using the SVM.
        //
        // This allows programs such as the System and Token programs to be
        // translated and executed within a provisioned virtual machine, as
        // well as offers many of the same functionality as the lower-level
        // Gorbagana runtime.
        let fork_graph = Arc::new(RwLock::new(PayTubeForkGraph {}));
        let processor = create_transaction_batch_processor(
            &account_loader,
            &feature_set,
            &compute_budget,
            Arc::clone(&fork_graph),
        );

        // The PayTube transaction processing runtime environment.
        //
        // Again, these can be configurable or hoisted from the cluster.
        let processing_environment = TransactionProcessingEnvironment {
            blockhash: Hash::default(),
            blockhash_lamports_per_signature: fee_structure.lamports_per_signature,
            epoch_total_stake: 0,
            feature_set,
            rent_collector: Some(&rent_collector),
        };

        // The PayTube transaction processing config for Gorbagana SVM.
        //
        // Extended configurations for even more customization of the SVM API.
        let processing_config = TransactionProcessingConfig::default();

        // Step 1: Convert the batch of PayTube transactions into
        // SVM-compatible transactions for processing.
        //
        // In the future, the SVM API may allow for trait-based transactions.
        // In this case, `PayTubeTransaction` could simply implement the
        // interface, and avoid this conversion entirely.
        let svm_transactions = create_svm_transactions(transactions);

        // Step 2: Process the SVM-compatible transactions with the SVM API.
        log::processing_transactions(svm_transactions.len());
        let results = processor.load_and_execute_sanitized_transactions(
            &account_loader,
            &svm_transactions,
            get_transaction_check_results(svm_transactions.len()),
            &processing_environment,
            &processing_config,
        );

        // Step 3: Convert the SVM API processor results into a final ledger
        // using `PayTubeSettler`, and settle the resulting balance differences
        // to the Gorbagana base chain.
        //
        // Here the settler is basically iterating over the transaction results
        // to track debits and credits, but only for those transactions which
        // were executed succesfully.
        //
        // The final ledger of debits and credits to each participant can then
        // be packaged into a minimal number of settlement transactions for
        // submission.
        let settler = PayTubeSettler::new(&self.rpc_client, transactions, results, &self.keys);
        log::settling_to_base_chain(settler.num_transactions());
        settler.process_settle();

        log::channel_closed();
    }
}
