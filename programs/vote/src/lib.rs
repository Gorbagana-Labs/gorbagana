#![cfg_attr(feature = "frozen-abi", feature(min_specialization))]

pub mod vote_processor;
pub mod vote_state;

#[cfg_attr(feature = "metrics", macro_use)]
#[cfg(feature = "metrics")]
extern crate gorbagana_metrics;

#[cfg(feature = "frozen-abi")]
extern crate gorbagana_frozen_abi_macro;

pub use gorbagana_vote_interface::{
    authorized_voters, error as vote_error, instruction as vote_instruction,
    program::{check_id, id},
};
