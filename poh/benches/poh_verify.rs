#![feature(test)]
extern crate test;

use {
    gorbagana_entry::entry::{self, next_entry_mut, Entry, EntrySlice},
    gorbagana_hash::Hash,
    gorbagana_keypair::Keypair,
    gorbagana_sha256_hasher::hash,
    gorbagana_signer::Signer,
    gorbagana_system_transaction::transfer,
    test::Bencher,
};

const NUM_HASHES: u64 = 400;
const NUM_ENTRIES: usize = 800;

#[bench]
fn bench_poh_verify_ticks(bencher: &mut Bencher) {
    gorbagana_logger::setup();
    let thread_pool = entry::thread_pool_for_benches();

    let zero = Hash::default();
    let start_hash = hash(zero.as_ref());
    let mut cur_hash = start_hash;

    let mut ticks: Vec<Entry> = Vec::with_capacity(NUM_ENTRIES);
    for _ in 0..NUM_ENTRIES {
        ticks.push(next_entry_mut(&mut cur_hash, NUM_HASHES, vec![]));
    }

    bencher.iter(|| {
        assert!(ticks.verify(&start_hash, &thread_pool));
    })
}

#[bench]
fn bench_poh_verify_transaction_entries(bencher: &mut Bencher) {
    let thread_pool = entry::thread_pool_for_benches();

    let zero = Hash::default();
    let start_hash = hash(zero.as_ref());
    let mut cur_hash = start_hash;

    let keypair1 = Keypair::new();
    let pubkey1 = keypair1.pubkey();

    let mut ticks: Vec<Entry> = Vec::with_capacity(NUM_ENTRIES);
    for _ in 0..NUM_ENTRIES {
        let tx = transfer(&keypair1, &pubkey1, 42, cur_hash);
        ticks.push(next_entry_mut(&mut cur_hash, NUM_HASHES, vec![tx]));
    }

    bencher.iter(|| {
        assert!(ticks.verify(&start_hash, &thread_pool));
    })
}
