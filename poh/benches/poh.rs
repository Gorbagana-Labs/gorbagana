// This bench attempts to justify the value of `gorbagana_core::poh_service::NUM_HASHES_PER_BATCH`

#![feature(test)]
extern crate test;

use {
    gorbagana_entry::poh::Poh,
    gorbagana_hash::Hash,
    gorbagana_ledger::{
        blockstore::Blockstore,
        genesis_utils::{create_genesis_config, GenesisConfigInfo},
        get_tmp_ledger_path_auto_delete,
        leader_schedule_cache::LeaderScheduleCache,
    },
    gorbagana_perf::test_tx::test_tx,
    gorbagana_poh::{poh_recorder::PohRecorder, poh_service::DEFAULT_HASHES_PER_BATCH},
    gorbagana_poh_config::PohConfig,
    gorbagana_runtime::bank::Bank,
    gorbagana_sha256_hasher::hash,
    gorbagana_transaction::sanitized::SanitizedTransaction,
    std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    test::Bencher,
};

const NUM_HASHES: u64 = 30_000; // Should require ~10ms on a 2017 MacBook Pro

#[bench]
// No locking.  Fastest.
fn bench_poh_hash(bencher: &mut Bencher) {
    let mut poh = Poh::new(Hash::default(), None);
    bencher.iter(|| {
        poh.hash(NUM_HASHES);
    })
}

#[bench]
// Lock on each iteration.  Slowest.
fn bench_arc_mutex_poh_hash(bencher: &mut Bencher) {
    let poh = Arc::new(Mutex::new(Poh::new(Hash::default(), None)));
    bencher.iter(|| {
        for _ in 0..NUM_HASHES {
            poh.lock().unwrap().hash(1);
        }
    })
}

#[bench]
// Acquire lock every NUM_HASHES_PER_BATCH iterations.
// Speed should be close to bench_poh_hash() if NUM_HASHES_PER_BATCH is set well.
fn bench_arc_mutex_poh_batched_hash(bencher: &mut Bencher) {
    let poh = Arc::new(Mutex::new(Poh::new(Hash::default(), Some(NUM_HASHES))));
    //let exit = Arc::new(AtomicBool::new(false));
    let exit = Arc::new(AtomicBool::new(true));

    bencher.iter(|| {
        // NOTE: This block attempts to look as close as possible to `PohService::tick_producer()`
        loop {
            if poh.lock().unwrap().hash(DEFAULT_HASHES_PER_BATCH) {
                poh.lock().unwrap().tick().unwrap();
                if exit.load(Ordering::Relaxed) {
                    break;
                }
            }
        }
    })
}

#[bench]
// Worst case transaction record delay due to batch hashing at NUM_HASHES_PER_BATCH
fn bench_poh_lock_time_per_batch(bencher: &mut Bencher) {
    let mut poh = Poh::new(Hash::default(), None);
    bencher.iter(|| {
        poh.hash(DEFAULT_HASHES_PER_BATCH);
    })
}

#[bench]
fn bench_poh_recorder_record_transaction_index(bencher: &mut Bencher) {
    let ledger_path = get_tmp_ledger_path_auto_delete!();
    let blockstore =
        Blockstore::open(ledger_path.path()).expect("Expected to be able to open database ledger");
    let GenesisConfigInfo { genesis_config, .. } = create_genesis_config(2);
    let bank = Arc::new(Bank::new_for_tests(&genesis_config));
    let prev_hash = bank.last_blockhash();

    let (mut poh_recorder, _entry_receiver) = PohRecorder::new(
        0,
        prev_hash,
        bank.clone(),
        Some((4, 4)),
        bank.ticks_per_slot(),
        Arc::new(blockstore),
        &std::sync::Arc::new(LeaderScheduleCache::new_from_bank(&bank)),
        &PohConfig::default(),
        Arc::new(AtomicBool::default()),
    );
    let h1 = hash(b"hello Agave, hello Anza!");

    poh_recorder.set_bank_with_transaction_index_for_test(bank.clone());
    poh_recorder.tick();
    let txs: [SanitizedTransaction; 7] = [
        SanitizedTransaction::from_transaction_for_tests(test_tx()),
        SanitizedTransaction::from_transaction_for_tests(test_tx()),
        SanitizedTransaction::from_transaction_for_tests(test_tx()),
        SanitizedTransaction::from_transaction_for_tests(test_tx()),
        SanitizedTransaction::from_transaction_for_tests(test_tx()),
        SanitizedTransaction::from_transaction_for_tests(test_tx()),
        SanitizedTransaction::from_transaction_for_tests(test_tx()),
    ];

    let txs: Vec<_> = txs.iter().map(|tx| tx.to_versioned_transaction()).collect();
    bencher.iter(|| {
        let _record_result = poh_recorder
            .record(
                bank.slot(),
                vec![test::black_box(h1)],
                vec![test::black_box(txs.clone())],
            )
            .unwrap()
            .unwrap();
    });
    poh_recorder.tick();
}

#[bench]
fn bench_poh_recorder_set_bank(bencher: &mut Bencher) {
    let ledger_path = get_tmp_ledger_path_auto_delete!();
    let blockstore =
        Blockstore::open(ledger_path.path()).expect("Expected to be able to open database ledger");
    let GenesisConfigInfo { genesis_config, .. } = create_genesis_config(2);
    let bank = Arc::new(Bank::new_for_tests(&genesis_config));
    let prev_hash = bank.last_blockhash();

    let (mut poh_recorder, _entry_receiver) = PohRecorder::new(
        0,
        prev_hash,
        bank.clone(),
        Some((4, 4)),
        bank.ticks_per_slot(),
        Arc::new(blockstore),
        &std::sync::Arc::new(LeaderScheduleCache::new_from_bank(&bank)),
        &PohConfig::default(),
        Arc::new(AtomicBool::default()),
    );
    bencher.iter(|| {
        poh_recorder.set_bank_with_transaction_index_for_test(bank.clone());
        poh_recorder.tick();
        poh_recorder.clear_bank_for_test();
    });
}
