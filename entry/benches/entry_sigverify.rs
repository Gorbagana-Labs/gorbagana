#![feature(test)]
extern crate test;
use {
    agave_reserved_account_keys::ReservedAccountKeys,
    gorbagana_entry::entry::{self, VerifyRecyclers},
    gorbagana_hash::Hash,
    gorbagana_message::SimpleAddressLoader,
    gorbagana_perf::test_tx::test_tx,
    gorbagana_runtime_transaction::runtime_transaction::RuntimeTransaction,
    gorbagana_transaction::{
        sanitized::{MessageHash, SanitizedTransaction},
        versioned::VersionedTransaction,
        TransactionVerificationMode,
    },
    gorbagana_transaction_error::TransactionResult as Result,
    std::sync::Arc,
    test::Bencher,
};

#[bench]
fn bench_gpusigverify(bencher: &mut Bencher) {
    let thread_pool = entry::thread_pool_for_benches();
    let entries = (0..131072)
        .map(|_| {
            let transaction = test_tx();
            entry::next_entry_mut(&mut Hash::default(), 0, vec![transaction])
        })
        .collect::<Vec<_>>();

    let verify_transaction = {
        move |versioned_tx: VersionedTransaction,
              verification_mode: TransactionVerificationMode|
              -> Result<RuntimeTransaction<SanitizedTransaction>> {
            let sanitized_tx = {
                let message_hash =
                    if verification_mode == TransactionVerificationMode::FullVerification {
                        versioned_tx.verify_and_hash_message()?
                    } else {
                        versioned_tx.message.hash()
                    };

                RuntimeTransaction::try_create(
                    versioned_tx,
                    MessageHash::Precomputed(message_hash),
                    None,
                    SimpleAddressLoader::Disabled,
                    &ReservedAccountKeys::empty_key_set(),
                )
            }?;

            Ok(sanitized_tx)
        }
    };

    let recycler = VerifyRecyclers::default();

    bencher.iter(|| {
        let res = entry::start_verify_transactions(
            entries.clone(),
            false,
            &thread_pool,
            recycler.clone(),
            Arc::new(verify_transaction),
        );

        if let Ok(mut res) = res {
            let _ans = res.finish_verify();
        }
    })
}

#[bench]
fn bench_cpusigverify(bencher: &mut Bencher) {
    let thread_pool = entry::thread_pool_for_benches();
    let entries = (0..131072)
        .map(|_| {
            let transaction = test_tx();
            entry::next_entry_mut(&mut Hash::default(), 0, vec![transaction])
        })
        .collect::<Vec<_>>();

    let verify_transaction = {
        move |versioned_tx: VersionedTransaction| -> Result<RuntimeTransaction<SanitizedTransaction>> {
            let sanitized_tx = {
                let message_hash = versioned_tx.verify_and_hash_message()?;
                RuntimeTransaction::try_create(
                    versioned_tx,
                    MessageHash::Precomputed(message_hash),
                    None,
                    SimpleAddressLoader::Disabled,
                    &ReservedAccountKeys::empty_key_set(),
                )
            }?;

            Ok(sanitized_tx)
        }
    };

    bencher.iter(|| {
        let _ans =
            entry::verify_transactions(entries.clone(), &thread_pool, Arc::new(verify_transaction));
    })
}
