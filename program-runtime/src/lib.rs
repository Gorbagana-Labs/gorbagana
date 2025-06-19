#![cfg_attr(feature = "frozen-abi", feature(min_specialization))]
#![deny(clippy::arithmetic_side_effects)]
#![deny(clippy::indexing_slicing)]

#[cfg(feature = "metrics")]
#[macro_use]
extern crate gorbagana_metrics;

pub use gorbagana_sbpf;
pub mod execution_budget;
pub mod invoke_context;
pub mod loaded_programs;
pub mod mem_pool;
pub mod serialization;
pub mod stable_log;
pub mod sysvar_cache;

// re-exports for macros
pub mod __private {
    pub use {
        gorbagana_account::ReadableAccount, gorbagana_hash::Hash,
        gorbagana_instruction::error::InstructionError, gorbagana_rent::Rent,
        gorbagana_transaction_context::TransactionContext,
    };
}
