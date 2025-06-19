#![allow(clippy::arithmetic_side_effects)]
pub mod system_instruction;
pub mod system_processor;

use gorbagana_sdk_ids::system_program;
pub use {
    gorbagana_nonce_account::{get_system_account_kind, SystemAccountKind},
    system_program::id,
};
