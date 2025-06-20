#![allow(clippy::arithmetic_side_effects)]
#![feature(test)]

use {
    rayon::{
        iter::IndexedParallelIterator,
        prelude::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator},
    },
    gorbagana_account::{Account, ReadableAccount},
    gorbagana_clock::Epoch,
    gorbagana_keypair::Keypair,
    gorbagana_ledger::{
        blockstore_processor::{execute_batch, TransactionBatchWithIndexes},
        genesis_utils::{create_genesis_config, GenesisConfigInfo},
    },
    gorbagana_runtime::{
        bank::Bank,
        bank_forks::BankForks,
        prioritization_fee_cache::PrioritizationFeeCache,
        transaction_batch::{OwnedOrBorrowed, TransactionBatch},
    },
    gorbagana_runtime_transaction::runtime_transaction::RuntimeTransaction,
    gorbagana_signer::Signer,
    gorbagana_system_interface::program as system_program,
    gorbagana_system_transaction as system_transaction,
    gorbagana_timings::ExecuteTimings,
    gorbagana_transaction::sanitized::SanitizedTransaction,
    std::sync::{Arc, RwLock},
    test::Bencher,
};

extern crate test;

fn create_accounts(num: usize) -> Vec<Keypair> {
    (0..num).into_par_iter().map(|_| Keypair::new()).collect()
}

fn create_funded_accounts(bank: &Bank, num: usize) -> Vec<Keypair> {
    assert!(
        num.is_power_of_two(),
        "must be power of 2 for parallel funding tree"
    );
    let accounts = create_accounts(num);

    accounts.par_iter().for_each(|account| {
        bank.store_account(
            &account.pubkey(),
            &Account {
                lamports: 5100,
                data: vec![],
                owner: system_program::id(),
                executable: false,
                rent_epoch: Epoch::MAX,
            }
            .to_account_shared_data(),
        );
    });

    accounts
}

fn create_transactions(bank: &Bank, num: usize) -> Vec<RuntimeTransaction<SanitizedTransaction>> {
    let funded_accounts = create_funded_accounts(bank, 2 * num);
    funded_accounts
        .into_par_iter()
        .chunks(2)
        .map(|chunk| {
            let from = &chunk[0];
            let to = &chunk[1];
            system_transaction::transfer(from, &to.pubkey(), 1, bank.last_blockhash())
        })
        .map(RuntimeTransaction::from_transaction_for_tests)
        .collect()
}

struct BenchFrame {
    bank: Arc<Bank>,
    _bank_forks: Arc<RwLock<BankForks>>,
    prioritization_fee_cache: PrioritizationFeeCache,
}

fn setup() -> BenchFrame {
    let mint_total = u64::MAX;
    let GenesisConfigInfo {
        mut genesis_config, ..
    } = create_genesis_config(mint_total);

    // Set a high ticks_per_slot so we don't run out of ticks
    // during the benchmark
    genesis_config.ticks_per_slot = 10_000;

    let mut bank = Bank::new_for_benches(&genesis_config);

    // Allow arbitrary transaction processing time for the purposes of this bench
    bank.ns_per_slot = u128::MAX;

    // set cost tracker limits to MAX so it will not filter out TXs
    bank.write_cost_tracker()
        .unwrap()
        .set_limits(u64::MAX, u64::MAX, u64::MAX);
    let (bank, bank_forks) = bank.wrap_with_bank_forks_for_tests();
    let prioritization_fee_cache = PrioritizationFeeCache::default();
    BenchFrame {
        bank,
        _bank_forks: bank_forks,
        prioritization_fee_cache,
    }
}

fn bench_execute_batch(bencher: &mut Bencher, batch_size: usize) {
    const TRANSACTIONS_PER_ITERATION: usize = 64;
    assert_eq!(
        TRANSACTIONS_PER_ITERATION % batch_size,
        0,
        "batch_size must be a factor of `TRANSACTIONS_PER_ITERATION` \
         ({TRANSACTIONS_PER_ITERATION}) so that bench results are easily comparable"
    );
    let batches_per_iteration = TRANSACTIONS_PER_ITERATION / batch_size;

    let BenchFrame {
        bank,
        _bank_forks,
        prioritization_fee_cache,
    } = setup();
    let transactions = create_transactions(&bank, 2_usize.pow(20));
    let batches: Vec<_> = transactions
        .chunks(batch_size)
        .map(|txs| {
            let mut batch = TransactionBatch::new(
                vec![Ok(()); txs.len()],
                &bank,
                OwnedOrBorrowed::Borrowed(txs),
            );
            batch.set_needs_unlock(false);
            TransactionBatchWithIndexes {
                batch,
                transaction_indexes: (0..batch_size).collect(),
            }
        })
        .collect();
    let mut batches_iter = batches.iter();

    let mut timing = ExecuteTimings::default();
    bencher.iter(|| {
        for _ in 0..batches_per_iteration {
            let batch = batches_iter.next().unwrap();
            let _ = execute_batch(
                batch,
                &bank,
                None,
                None,
                &mut timing,
                None,
                &prioritization_fee_cache,
                None::<fn(&_) -> _>,
            );
        }
    });
    // drop batches here so dropping is not included in the benchmark
    drop(batches);
}

#[bench]
fn bench_execute_batch_unbatched(bencher: &mut Bencher) {
    bench_execute_batch(bencher, 1);
}

#[bench]
fn bench_execute_batch_half_batch(bencher: &mut Bencher) {
    bench_execute_batch(bencher, 32);
}

#[bench]
fn bench_execute_batch_full_batch(bencher: &mut Bencher) {
    bench_execute_batch(bencher, 64);
}
