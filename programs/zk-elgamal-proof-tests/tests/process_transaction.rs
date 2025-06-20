use {
    bytemuck::{bytes_of, Pod},
    gorbagana_account::Account,
    gorbagana_instruction::error::InstructionError,
    gorbagana_keypair::Keypair,
    gorbagana_program_test::*,
    gorbagana_pubkey::Pubkey,
    gorbagana_signer::Signer,
    gorbagana_system_interface::instruction as system_instruction,
    gorbagana_transaction::Transaction,
    gorbagana_transaction_error::TransactionError,
    gorbagana_zk_sdk::{
        encryption::{
            elgamal::{ElGamalKeypair, ElGamalSecretKey},
            grouped_elgamal::GroupedElGamal,
            pedersen::{Pedersen, PedersenOpening},
        },
        zk_elgamal_proof_program::{self, instruction::*, proof_data::*, state::ProofContextState},
    },
    std::mem::size_of,
};

const VERIFY_INSTRUCTION_TYPES: [ProofInstruction; 12] = [
    ProofInstruction::VerifyZeroCiphertext,
    ProofInstruction::VerifyCiphertextCiphertextEquality,
    ProofInstruction::VerifyPubkeyValidity,
    ProofInstruction::VerifyBatchedRangeProofU64,
    ProofInstruction::VerifyBatchedRangeProofU128,
    ProofInstruction::VerifyBatchedRangeProofU256,
    ProofInstruction::VerifyCiphertextCommitmentEquality,
    ProofInstruction::VerifyGroupedCiphertext2HandlesValidity,
    ProofInstruction::VerifyBatchedGroupedCiphertext2HandlesValidity,
    ProofInstruction::VerifyPercentageWithCap,
    ProofInstruction::VerifyGroupedCiphertext3HandlesValidity,
    ProofInstruction::VerifyBatchedGroupedCiphertext3HandlesValidity,
];

#[tokio::test]
async fn test_zero_balance() {
    let elgamal_keypair = ElGamalKeypair::new_rand();

    let zero_ciphertext = elgamal_keypair.pubkey().encrypt(0_u64);
    let success_proof_data =
        ZeroCiphertextProofData::new(&elgamal_keypair, &zero_ciphertext).unwrap();

    let incorrect_pubkey = elgamal_keypair.pubkey();
    let incorrect_secret = ElGamalSecretKey::new_rand();
    let incorrect_keypair = ElGamalKeypair::new_for_tests(*incorrect_pubkey, incorrect_secret);

    let fail_proof_data =
        ZeroCiphertextProofData::new(&incorrect_keypair, &zero_ciphertext).unwrap();

    test_verify_proof_without_context(
        ProofInstruction::VerifyZeroCiphertext,
        &success_proof_data,
        &fail_proof_data,
    )
    .await;

    test_verify_proof_from_account_with_context(
        ProofInstruction::VerifyZeroCiphertext,
        size_of::<ProofContextState<ZeroCiphertextProofContext>>(),
        &success_proof_data,
        &fail_proof_data,
    )
    .await;

    test_verify_proof_with_context(
        ProofInstruction::VerifyZeroCiphertext,
        size_of::<ProofContextState<ZeroCiphertextProofContext>>(),
        &success_proof_data,
        &fail_proof_data,
    )
    .await;

    test_close_context_state(
        ProofInstruction::VerifyZeroCiphertext,
        size_of::<ProofContextState<ZeroCiphertextProofContext>>(),
        &success_proof_data,
    )
    .await;
}

#[tokio::test]
async fn test_ciphertext_ciphertext_equality() {
    let source_keypair = ElGamalKeypair::new_rand();
    let destination_keypair = ElGamalKeypair::new_rand();

    let amount: u64 = 0;
    let source_ciphertext = source_keypair.pubkey().encrypt(amount);

    let destination_opening = PedersenOpening::new_rand();
    let destination_ciphertext = destination_keypair
        .pubkey()
        .encrypt_with(amount, &destination_opening);

    let success_proof_data = CiphertextCiphertextEqualityProofData::new(
        &source_keypair,
        destination_keypair.pubkey(),
        &source_ciphertext,
        &destination_ciphertext,
        &destination_opening,
        amount,
    )
    .unwrap();

    let incorrect_pubkey = source_keypair.pubkey();
    let incorrect_secret = ElGamalSecretKey::new_rand();
    let incorrect_keypair = ElGamalKeypair::new_for_tests(*incorrect_pubkey, incorrect_secret);

    let fail_proof_data = CiphertextCiphertextEqualityProofData::new(
        &incorrect_keypair,
        destination_keypair.pubkey(),
        &source_ciphertext,
        &destination_ciphertext,
        &destination_opening,
        amount,
    )
    .unwrap();

    test_verify_proof_without_context(
        ProofInstruction::VerifyCiphertextCiphertextEquality,
        &success_proof_data,
        &fail_proof_data,
    )
    .await;

    test_verify_proof_with_context(
        ProofInstruction::VerifyCiphertextCiphertextEquality,
        size_of::<ProofContextState<CiphertextCiphertextEqualityProofContext>>(),
        &success_proof_data,
        &fail_proof_data,
    )
    .await;

    test_verify_proof_from_account_with_context(
        ProofInstruction::VerifyCiphertextCiphertextEquality,
        size_of::<ProofContextState<CiphertextCiphertextEqualityProofContext>>(),
        &success_proof_data,
        &fail_proof_data,
    )
    .await;
    test_close_context_state(
        ProofInstruction::VerifyCiphertextCiphertextEquality,
        size_of::<ProofContextState<CiphertextCiphertextEqualityProofContext>>(),
        &success_proof_data,
    )
    .await;
}

#[tokio::test]
async fn test_pubkey_validity() {
    let elgamal_keypair = ElGamalKeypair::new_rand();

    let success_proof_data = PubkeyValidityProofData::new(&elgamal_keypair).unwrap();

    let incorrect_pubkey = elgamal_keypair.pubkey();
    let incorrect_secret = ElGamalSecretKey::new_rand();
    let incorrect_keypair = ElGamalKeypair::new_for_tests(*incorrect_pubkey, incorrect_secret);

    let fail_proof_data = PubkeyValidityProofData::new(&incorrect_keypair).unwrap();

    test_verify_proof_without_context(
        ProofInstruction::VerifyPubkeyValidity,
        &success_proof_data,
        &fail_proof_data,
    )
    .await;

    test_verify_proof_with_context(
        ProofInstruction::VerifyPubkeyValidity,
        size_of::<ProofContextState<PubkeyValidityProofContext>>(),
        &success_proof_data,
        &fail_proof_data,
    )
    .await;

    test_verify_proof_from_account_with_context(
        ProofInstruction::VerifyPubkeyValidity,
        size_of::<ProofContextState<PubkeyValidityProofContext>>(),
        &success_proof_data,
        &fail_proof_data,
    )
    .await;

    test_close_context_state(
        ProofInstruction::VerifyPubkeyValidity,
        size_of::<ProofContextState<PubkeyValidityProofContext>>(),
        &success_proof_data,
    )
    .await;
}

#[tokio::test]
async fn test_batched_range_proof_u64() {
    let amount_1 = 23_u64;
    let amount_2 = 24_u64;

    let (commitment_1, opening_1) = Pedersen::new(amount_1);
    let (commitment_2, opening_2) = Pedersen::new(amount_2);

    let success_proof_data = BatchedRangeProofU64Data::new(
        vec![&commitment_1, &commitment_2],
        vec![amount_1, amount_2],
        vec![32, 32],
        vec![&opening_1, &opening_2],
    )
    .unwrap();

    let incorrect_opening = PedersenOpening::new_rand();
    let fail_proof_data = BatchedRangeProofU64Data::new(
        vec![&commitment_1, &commitment_2],
        vec![amount_1, amount_2],
        vec![32, 32],
        vec![&opening_1, &incorrect_opening],
    )
    .unwrap();

    test_verify_proof_without_context(
        ProofInstruction::VerifyBatchedRangeProofU64,
        &success_proof_data,
        &fail_proof_data,
    )
    .await;

    test_verify_proof_with_context(
        ProofInstruction::VerifyBatchedRangeProofU64,
        size_of::<ProofContextState<BatchedRangeProofContext>>(),
        &success_proof_data,
        &fail_proof_data,
    )
    .await;

    test_verify_proof_from_account_with_context(
        ProofInstruction::VerifyBatchedRangeProofU64,
        size_of::<ProofContextState<BatchedRangeProofContext>>(),
        &success_proof_data,
        &fail_proof_data,
    )
    .await;

    test_close_context_state(
        ProofInstruction::VerifyBatchedRangeProofU64,
        size_of::<ProofContextState<BatchedRangeProofContext>>(),
        &success_proof_data,
    )
    .await;
}

#[tokio::test]
async fn test_batched_range_proof_u128() {
    let amount_1 = 23_u64;
    let amount_2 = 24_u64;

    let (commitment_1, opening_1) = Pedersen::new(amount_1);
    let (commitment_2, opening_2) = Pedersen::new(amount_2);

    let success_proof_data = BatchedRangeProofU128Data::new(
        vec![&commitment_1, &commitment_2],
        vec![amount_1, amount_2],
        vec![64, 64],
        vec![&opening_1, &opening_2],
    )
    .unwrap();

    let incorrect_opening = PedersenOpening::new_rand();
    let fail_proof_data = BatchedRangeProofU128Data::new(
        vec![&commitment_1, &commitment_2],
        vec![amount_1, amount_2],
        vec![64, 64],
        vec![&opening_1, &incorrect_opening],
    )
    .unwrap();

    test_verify_proof_without_context(
        ProofInstruction::VerifyBatchedRangeProofU128,
        &success_proof_data,
        &fail_proof_data,
    )
    .await;

    test_verify_proof_with_context(
        ProofInstruction::VerifyBatchedRangeProofU128,
        size_of::<ProofContextState<BatchedRangeProofContext>>(),
        &success_proof_data,
        &fail_proof_data,
    )
    .await;

    test_verify_proof_from_account_with_context(
        ProofInstruction::VerifyBatchedRangeProofU128,
        size_of::<ProofContextState<BatchedRangeProofContext>>(),
        &success_proof_data,
        &fail_proof_data,
    )
    .await;

    test_close_context_state(
        ProofInstruction::VerifyBatchedRangeProofU128,
        size_of::<ProofContextState<BatchedRangeProofContext>>(),
        &success_proof_data,
    )
    .await;
}

#[tokio::test]
async fn test_batched_range_proof_u256() {
    let amount_1 = 23_u64;
    let amount_2 = 24_u64;
    let amount_3 = 25_u64;
    let amount_4 = 26_u64;

    let (commitment_1, opening_1) = Pedersen::new(amount_1);
    let (commitment_2, opening_2) = Pedersen::new(amount_2);
    let (commitment_3, opening_3) = Pedersen::new(amount_3);
    let (commitment_4, opening_4) = Pedersen::new(amount_4);

    let success_proof_data = BatchedRangeProofU256Data::new(
        vec![&commitment_1, &commitment_2, &commitment_3, &commitment_4],
        vec![amount_1, amount_2, amount_3, amount_4],
        vec![64, 64, 64, 64],
        vec![&opening_1, &opening_2, &opening_3, &opening_4],
    )
    .unwrap();

    let incorrect_opening = PedersenOpening::new_rand();
    let fail_proof_data = BatchedRangeProofU256Data::new(
        vec![&commitment_1, &commitment_2, &commitment_3, &commitment_4],
        vec![amount_1, amount_2, amount_3, amount_4],
        vec![64, 64, 64, 64],
        vec![&opening_1, &opening_2, &opening_3, &incorrect_opening],
    )
    .unwrap();

    test_verify_proof_without_context(
        ProofInstruction::VerifyBatchedRangeProofU256,
        &success_proof_data,
        &fail_proof_data,
    )
    .await;

    test_verify_proof_with_context(
        ProofInstruction::VerifyBatchedRangeProofU256,
        size_of::<ProofContextState<BatchedRangeProofContext>>(),
        &success_proof_data,
        &fail_proof_data,
    )
    .await;

    test_verify_proof_from_account_with_context(
        ProofInstruction::VerifyBatchedRangeProofU256,
        size_of::<ProofContextState<BatchedRangeProofContext>>(),
        &success_proof_data,
        &fail_proof_data,
    )
    .await;

    test_close_context_state(
        ProofInstruction::VerifyBatchedRangeProofU256,
        size_of::<ProofContextState<BatchedRangeProofContext>>(),
        &success_proof_data,
    )
    .await;
}

#[tokio::test]
async fn test_ciphertext_commitment_equality() {
    let keypair = ElGamalKeypair::new_rand();
    let amount: u64 = 55;
    let ciphertext = keypair.pubkey().encrypt(amount);
    let (commitment, opening) = Pedersen::new(amount);

    let success_proof_data = CiphertextCommitmentEqualityProofData::new(
        &keypair,
        &ciphertext,
        &commitment,
        &opening,
        amount,
    )
    .unwrap();

    let incorrect_pubkey = keypair.pubkey();
    let incorrect_secret = ElGamalSecretKey::new_rand();
    let incorrect_keypair = ElGamalKeypair::new_for_tests(*incorrect_pubkey, incorrect_secret);

    let fail_proof_data = CiphertextCommitmentEqualityProofData::new(
        &incorrect_keypair,
        &ciphertext,
        &commitment,
        &opening,
        amount,
    )
    .unwrap();

    test_verify_proof_without_context(
        ProofInstruction::VerifyCiphertextCommitmentEquality,
        &success_proof_data,
        &fail_proof_data,
    )
    .await;

    test_verify_proof_with_context(
        ProofInstruction::VerifyCiphertextCommitmentEquality,
        size_of::<ProofContextState<CiphertextCommitmentEqualityProofContext>>(),
        &success_proof_data,
        &fail_proof_data,
    )
    .await;

    test_verify_proof_from_account_with_context(
        ProofInstruction::VerifyCiphertextCommitmentEquality,
        size_of::<ProofContextState<CiphertextCommitmentEqualityProofContext>>(),
        &success_proof_data,
        &fail_proof_data,
    )
    .await;

    test_close_context_state(
        ProofInstruction::VerifyCiphertextCommitmentEquality,
        size_of::<ProofContextState<CiphertextCommitmentEqualityProofContext>>(),
        &success_proof_data,
    )
    .await;
}

#[tokio::test]
async fn test_grouped_ciphertext_2_handles_validity() {
    let destination_keypair = ElGamalKeypair::new_rand();
    let destination_pubkey = destination_keypair.pubkey();

    let auditor_keypair = ElGamalKeypair::new_rand();
    let auditor_pubkey = auditor_keypair.pubkey();

    let amount: u64 = 55;
    let opening = PedersenOpening::new_rand();
    let grouped_ciphertext =
        GroupedElGamal::encrypt_with([destination_pubkey, auditor_pubkey], amount, &opening);

    let success_proof_data = GroupedCiphertext2HandlesValidityProofData::new(
        destination_pubkey,
        auditor_pubkey,
        &grouped_ciphertext,
        amount,
        &opening,
    )
    .unwrap();

    let incorrect_opening = PedersenOpening::new_rand();
    let fail_proof_data = GroupedCiphertext2HandlesValidityProofData::new(
        destination_pubkey,
        auditor_pubkey,
        &grouped_ciphertext,
        amount,
        &incorrect_opening,
    )
    .unwrap();

    test_verify_proof_without_context(
        ProofInstruction::VerifyGroupedCiphertext2HandlesValidity,
        &success_proof_data,
        &fail_proof_data,
    )
    .await;

    test_verify_proof_with_context(
        ProofInstruction::VerifyGroupedCiphertext2HandlesValidity,
        size_of::<ProofContextState<GroupedCiphertext2HandlesValidityProofContext>>(),
        &success_proof_data,
        &fail_proof_data,
    )
    .await;

    test_verify_proof_from_account_with_context(
        ProofInstruction::VerifyGroupedCiphertext2HandlesValidity,
        size_of::<ProofContextState<GroupedCiphertext2HandlesValidityProofContext>>(),
        &success_proof_data,
        &fail_proof_data,
    )
    .await;

    test_close_context_state(
        ProofInstruction::VerifyGroupedCiphertext2HandlesValidity,
        size_of::<ProofContextState<GroupedCiphertext2HandlesValidityProofContext>>(),
        &success_proof_data,
    )
    .await;
}

#[tokio::test]
async fn test_batched_grouped_ciphertext_2_handles_validity() {
    let destination_keypair = ElGamalKeypair::new_rand();
    let destination_pubkey = destination_keypair.pubkey();

    let auditor_keypair = ElGamalKeypair::new_rand();
    let auditor_pubkey = auditor_keypair.pubkey();

    let amount_lo: u64 = 55;
    let amount_hi: u64 = 22;

    let opening_lo = PedersenOpening::new_rand();
    let opening_hi = PedersenOpening::new_rand();

    let grouped_ciphertext_lo =
        GroupedElGamal::encrypt_with([destination_pubkey, auditor_pubkey], amount_lo, &opening_lo);
    let grouped_ciphertext_hi =
        GroupedElGamal::encrypt_with([destination_pubkey, auditor_pubkey], amount_hi, &opening_hi);

    let success_proof_data = BatchedGroupedCiphertext2HandlesValidityProofData::new(
        destination_pubkey,
        auditor_pubkey,
        &grouped_ciphertext_lo,
        &grouped_ciphertext_hi,
        amount_lo,
        amount_hi,
        &opening_lo,
        &opening_hi,
    )
    .unwrap();

    let incorrect_opening = PedersenOpening::new_rand();
    let fail_proof_data = BatchedGroupedCiphertext2HandlesValidityProofData::new(
        destination_pubkey,
        auditor_pubkey,
        &grouped_ciphertext_lo,
        &grouped_ciphertext_hi,
        amount_lo,
        amount_hi,
        &incorrect_opening,
        &opening_hi,
    )
    .unwrap();

    test_verify_proof_without_context(
        ProofInstruction::VerifyBatchedGroupedCiphertext2HandlesValidity,
        &success_proof_data,
        &fail_proof_data,
    )
    .await;

    test_verify_proof_with_context(
        ProofInstruction::VerifyBatchedGroupedCiphertext2HandlesValidity,
        size_of::<ProofContextState<BatchedGroupedCiphertext2HandlesValidityProofContext>>(),
        &success_proof_data,
        &fail_proof_data,
    )
    .await;

    test_verify_proof_from_account_with_context(
        ProofInstruction::VerifyBatchedGroupedCiphertext2HandlesValidity,
        size_of::<ProofContextState<BatchedGroupedCiphertext2HandlesValidityProofContext>>(),
        &success_proof_data,
        &fail_proof_data,
    )
    .await;

    test_close_context_state(
        ProofInstruction::VerifyBatchedGroupedCiphertext2HandlesValidity,
        size_of::<ProofContextState<BatchedGroupedCiphertext2HandlesValidityProofContext>>(),
        &success_proof_data,
    )
    .await;
}

#[tokio::test]
async fn test_grouped_ciphertext_3_handles_validity() {
    let source_keypair = ElGamalKeypair::new_rand();
    let source_pubkey = source_keypair.pubkey();

    let destination_keypair = ElGamalKeypair::new_rand();
    let destination_pubkey = destination_keypair.pubkey();

    let auditor_keypair = ElGamalKeypair::new_rand();
    let auditor_pubkey = auditor_keypair.pubkey();

    let amount: u64 = 55;
    let opening = PedersenOpening::new_rand();
    let grouped_ciphertext = GroupedElGamal::encrypt_with(
        [source_pubkey, destination_pubkey, auditor_pubkey],
        amount,
        &opening,
    );

    let success_proof_data = GroupedCiphertext3HandlesValidityProofData::new(
        source_pubkey,
        destination_pubkey,
        auditor_pubkey,
        &grouped_ciphertext,
        amount,
        &opening,
    )
    .unwrap();

    let incorrect_opening = PedersenOpening::new_rand();
    let fail_proof_data = GroupedCiphertext3HandlesValidityProofData::new(
        source_pubkey,
        destination_pubkey,
        auditor_pubkey,
        &grouped_ciphertext,
        amount,
        &incorrect_opening,
    )
    .unwrap();

    test_verify_proof_without_context(
        ProofInstruction::VerifyGroupedCiphertext3HandlesValidity,
        &success_proof_data,
        &fail_proof_data,
    )
    .await;

    test_verify_proof_with_context(
        ProofInstruction::VerifyGroupedCiphertext3HandlesValidity,
        size_of::<ProofContextState<GroupedCiphertext3HandlesValidityProofContext>>(),
        &success_proof_data,
        &fail_proof_data,
    )
    .await;

    test_close_context_state(
        ProofInstruction::VerifyGroupedCiphertext3HandlesValidity,
        size_of::<ProofContextState<GroupedCiphertext3HandlesValidityProofContext>>(),
        &success_proof_data,
    )
    .await;
}

#[tokio::test]
async fn test_batched_grouped_ciphertext_3_handles_validity() {
    let source_keypair = ElGamalKeypair::new_rand();
    let source_pubkey = source_keypair.pubkey();

    let destination_keypair = ElGamalKeypair::new_rand();
    let destination_pubkey = destination_keypair.pubkey();

    let auditor_keypair = ElGamalKeypair::new_rand();
    let auditor_pubkey = auditor_keypair.pubkey();

    let amount_lo: u64 = 55;
    let amount_hi: u64 = 22;

    let opening_lo = PedersenOpening::new_rand();
    let opening_hi = PedersenOpening::new_rand();

    let grouped_ciphertext_lo = GroupedElGamal::encrypt_with(
        [source_pubkey, destination_pubkey, auditor_pubkey],
        amount_lo,
        &opening_lo,
    );
    let grouped_ciphertext_hi = GroupedElGamal::encrypt_with(
        [source_pubkey, destination_pubkey, auditor_pubkey],
        amount_hi,
        &opening_hi,
    );

    let success_proof_data = BatchedGroupedCiphertext3HandlesValidityProofData::new(
        source_pubkey,
        destination_pubkey,
        auditor_pubkey,
        &grouped_ciphertext_lo,
        &grouped_ciphertext_hi,
        amount_lo,
        amount_hi,
        &opening_lo,
        &opening_hi,
    )
    .unwrap();

    let incorrect_opening = PedersenOpening::new_rand();
    let fail_proof_data = BatchedGroupedCiphertext3HandlesValidityProofData::new(
        source_pubkey,
        destination_pubkey,
        auditor_pubkey,
        &grouped_ciphertext_lo,
        &grouped_ciphertext_hi,
        amount_lo,
        amount_hi,
        &incorrect_opening,
        &opening_hi,
    )
    .unwrap();

    test_verify_proof_without_context(
        ProofInstruction::VerifyBatchedGroupedCiphertext3HandlesValidity,
        &success_proof_data,
        &fail_proof_data,
    )
    .await;

    test_verify_proof_with_context(
        ProofInstruction::VerifyBatchedGroupedCiphertext3HandlesValidity,
        size_of::<ProofContextState<BatchedGroupedCiphertext3HandlesValidityProofContext>>(),
        &success_proof_data,
        &fail_proof_data,
    )
    .await;

    test_close_context_state(
        ProofInstruction::VerifyBatchedGroupedCiphertext3HandlesValidity,
        size_of::<ProofContextState<BatchedGroupedCiphertext3HandlesValidityProofContext>>(),
        &success_proof_data,
    )
    .await;
}

async fn test_verify_proof_without_context<T, U>(
    proof_instruction: ProofInstruction,
    success_proof_data: &T,
    fail_proof_data: &T,
) where
    T: Pod + ZkProofData<U>,
    U: Pod,
{
    let mut program_test = ProgramTest::default();
    // the most expensive proof is the transfer-with-fee proof with 407_000 CUs
    program_test.set_compute_max_units(500_000);
    let success_proof_account = Pubkey::new_unique();
    program_test.add_account(
        success_proof_account,
        Account {
            lamports: 1_000_000_000,
            data: bytes_of(success_proof_data).to_vec(),
            owner: Pubkey::new_unique(),
            ..Account::default()
        },
    );
    let fail_proof_account = Pubkey::new_unique();
    program_test.add_account(
        fail_proof_account,
        Account {
            lamports: 1_000_000_000,
            data: bytes_of(fail_proof_data).to_vec(),
            owner: Pubkey::new_unique(),
            ..Account::default()
        },
    );
    let mut context = program_test.start_with_context().await;

    let client = &mut context.banks_client;
    let payer = &context.payer;

    // verify a valid proof (wihtout creating a context account)
    let instructions = vec![proof_instruction.encode_verify_proof(None, success_proof_data)];
    let transaction = Transaction::new_signed_with_payer(
        &instructions.with_max_compute_unit_limit(),
        Some(&payer.pubkey()),
        &[payer],
        client.get_latest_blockhash().await.unwrap(),
    );
    client.process_transaction(transaction).await.unwrap();

    // try to verify an invalid proof (without creating a context account)
    let instructions = vec![proof_instruction.encode_verify_proof(None, fail_proof_data)];
    let transaction = Transaction::new_signed_with_payer(
        &instructions.with_max_compute_unit_limit(),
        Some(&payer.pubkey()),
        &[payer],
        client.get_latest_blockhash().await.unwrap(),
    );
    let err = client
        .process_transaction(transaction)
        .await
        .unwrap_err()
        .unwrap();
    assert_eq!(
        err,
        TransactionError::InstructionError(0, InstructionError::InvalidInstructionData)
    );

    // try to verify a valid proof, but with a wrong proof type
    for wrong_instruction_type in VERIFY_INSTRUCTION_TYPES {
        if proof_instruction == wrong_instruction_type {
            continue;
        }

        let instruction =
            vec![wrong_instruction_type.encode_verify_proof(None, success_proof_data)];
        let transaction = Transaction::new_signed_with_payer(
            &instruction.with_max_compute_unit_limit(),
            Some(&payer.pubkey()),
            &[payer],
            client.get_latest_blockhash().await.unwrap(),
        );
        let err = client
            .process_transaction(transaction)
            .await
            .unwrap_err()
            .unwrap();
        assert_eq!(
            err,
            TransactionError::InstructionError(0, InstructionError::InvalidInstructionData)
        );
    }

    // verify a valid proof from an account
    let instruction =
        vec![proof_instruction.encode_verify_proof_from_account(None, &success_proof_account, 0)];
    let transaction = Transaction::new_signed_with_payer(
        &instruction,
        Some(&payer.pubkey()),
        &[payer],
        client.get_latest_blockhash().await.unwrap(),
    );
    client.process_transaction(transaction).await.unwrap();

    // try to verify an invalid proof from an account
    let instruction =
        vec![proof_instruction.encode_verify_proof_from_account(None, &fail_proof_account, 0)];
    let transaction = Transaction::new_signed_with_payer(
        &instruction,
        Some(&payer.pubkey()),
        &[payer],
        client.get_latest_blockhash().await.unwrap(),
    );
    client
        .process_transaction(transaction)
        .await
        .unwrap_err()
        .unwrap();
    assert_eq!(
        err,
        TransactionError::InstructionError(0, InstructionError::InvalidInstructionData)
    )
}

async fn test_verify_proof_with_context<T, U>(
    instruction_type: ProofInstruction,
    space: usize,
    success_proof_data: &T,
    fail_proof_data: &T,
) where
    T: Pod + ZkProofData<U>,
    U: Pod,
{
    let mut program_test = ProgramTest::default();
    // the most expensive proof is the transfer-with-fee proof with 407_000 CUs
    program_test.set_compute_max_units(500_000);
    let mut context = program_test.start_with_context().await;
    let rent = context.banks_client.get_rent().await.unwrap();

    let client = &mut context.banks_client;
    let payer = &context.payer;

    let context_state_account = Keypair::new();
    let context_state_authority = Keypair::new();

    let context_state_info = ContextStateInfo {
        context_state_account: &context_state_account.pubkey(),
        context_state_authority: &context_state_authority.pubkey(),
    };

    // try to create proof context state with an invalid proof
    let instructions = vec![
        system_instruction::create_account(
            &payer.pubkey(),
            &context_state_account.pubkey(),
            rent.minimum_balance(space),
            space as u64,
            &zk_elgamal_proof_program::id(),
        ),
        instruction_type.encode_verify_proof(Some(context_state_info), fail_proof_data),
    ];
    let transaction = Transaction::new_signed_with_payer(
        &instructions.with_max_compute_unit_limit(),
        Some(&payer.pubkey()),
        &[payer, &context_state_account],
        client.get_latest_blockhash().await.unwrap(),
    );
    let err = client
        .process_transaction(transaction)
        .await
        .unwrap_err()
        .unwrap();
    assert_eq!(
        err,
        TransactionError::InstructionError(1, InstructionError::InvalidInstructionData)
    );

    // try to create proof context state with incorrect account data length
    let instructions = vec![
        system_instruction::create_account(
            &payer.pubkey(),
            &context_state_account.pubkey(),
            rent.minimum_balance(space),
            (space.checked_sub(1).unwrap()) as u64,
            &zk_elgamal_proof_program::id(),
        ),
        instruction_type.encode_verify_proof(Some(context_state_info), success_proof_data),
    ];
    let transaction = Transaction::new_signed_with_payer(
        &instructions.with_max_compute_unit_limit(),
        Some(&payer.pubkey()),
        &[payer, &context_state_account],
        client.get_latest_blockhash().await.unwrap(),
    );
    let err = client
        .process_transaction(transaction)
        .await
        .unwrap_err()
        .unwrap();
    assert_eq!(
        err,
        TransactionError::InstructionError(1, InstructionError::InvalidAccountData)
    );

    // try to create proof context state with insufficient rent
    let instructions = vec![
        system_instruction::create_account(
            &payer.pubkey(),
            &context_state_account.pubkey(),
            rent.minimum_balance(space).checked_sub(1).unwrap(),
            space as u64,
            &zk_elgamal_proof_program::id(),
        ),
        instruction_type.encode_verify_proof(Some(context_state_info), success_proof_data),
    ];
    let transaction = Transaction::new_signed_with_payer(
        &instructions.with_max_compute_unit_limit(),
        Some(&payer.pubkey()),
        &[payer, &context_state_account],
        client.get_latest_blockhash().await.unwrap(),
    );
    let err = client
        .process_transaction(transaction)
        .await
        .unwrap_err()
        .unwrap();
    assert_eq!(
        err,
        TransactionError::InsufficientFundsForRent { account_index: 1 },
    );

    // try to create proof context state with an invalid `ProofType`
    for wrong_instruction_type in VERIFY_INSTRUCTION_TYPES {
        if instruction_type == wrong_instruction_type {
            continue;
        }

        let instructions = vec![
            system_instruction::create_account(
                &payer.pubkey(),
                &context_state_account.pubkey(),
                rent.minimum_balance(space),
                space as u64,
                &zk_elgamal_proof_program::id(),
            ),
            wrong_instruction_type
                .encode_verify_proof(Some(context_state_info), success_proof_data),
        ];
        let transaction = Transaction::new_signed_with_payer(
            &instructions.with_max_compute_unit_limit(),
            Some(&payer.pubkey()),
            &[payer, &context_state_account],
            client.get_latest_blockhash().await.unwrap(),
        );
        let err = client
            .process_transaction(transaction)
            .await
            .unwrap_err()
            .unwrap();
        assert_eq!(
            err,
            TransactionError::InstructionError(1, InstructionError::InvalidInstructionData)
        );
    }

    // successfully create a proof context state
    let instructions = vec![
        system_instruction::create_account(
            &payer.pubkey(),
            &context_state_account.pubkey(),
            rent.minimum_balance(space),
            space as u64,
            &zk_elgamal_proof_program::id(),
        ),
        instruction_type.encode_verify_proof(Some(context_state_info), success_proof_data),
    ];
    let transaction = Transaction::new_signed_with_payer(
        &instructions.with_max_compute_unit_limit(),
        Some(&payer.pubkey()),
        &[payer, &context_state_account],
        client.get_latest_blockhash().await.unwrap(),
    );
    client.process_transaction(transaction).await.unwrap();

    // try overwriting the context state
    let instructions =
        vec![instruction_type.encode_verify_proof(Some(context_state_info), success_proof_data)];
    let transaction = Transaction::new_signed_with_payer(
        &instructions.with_max_compute_unit_limit(),
        Some(&payer.pubkey()),
        &[payer],
        client.get_latest_blockhash().await.unwrap(),
    );
    let err = client
        .process_transaction(transaction)
        .await
        .unwrap_err()
        .unwrap();
    assert_eq!(
        err,
        TransactionError::InstructionError(0, InstructionError::AccountAlreadyInitialized)
    );

    // self-owned context state account
    let context_state_account_and_authority = Keypair::new();
    let context_state_info = ContextStateInfo {
        context_state_account: &context_state_account_and_authority.pubkey(),
        context_state_authority: &context_state_account_and_authority.pubkey(),
    };

    let instructions = vec![
        system_instruction::create_account(
            &payer.pubkey(),
            &context_state_account_and_authority.pubkey(),
            rent.minimum_balance(space),
            space as u64,
            &zk_elgamal_proof_program::id(),
        ),
        instruction_type.encode_verify_proof(Some(context_state_info), success_proof_data),
    ];
    let transaction = Transaction::new_signed_with_payer(
        &instructions.with_max_compute_unit_limit(),
        Some(&payer.pubkey()),
        &[payer, &context_state_account_and_authority],
        client.get_latest_blockhash().await.unwrap(),
    );
    client.process_transaction(transaction).await.unwrap();
}

async fn test_verify_proof_from_account_with_context<T, U>(
    instruction_type: ProofInstruction,
    space: usize,
    success_proof_data: &T,
    fail_proof_data: &T,
) where
    T: Pod + ZkProofData<U>,
    U: Pod,
{
    let mut program_test = ProgramTest::default();
    // the most expensive proof is the transfer-with-fee proof with 407_000 CUs
    program_test.set_compute_max_units(500_000);
    let success_proof_account = Pubkey::new_unique();
    program_test.add_account(
        success_proof_account,
        Account {
            lamports: 1_000_000_000,
            data: bytes_of(success_proof_data).to_vec(),
            owner: Pubkey::new_unique(),
            ..Account::default()
        },
    );
    let fail_proof_account = Pubkey::new_unique();
    program_test.add_account(
        fail_proof_account,
        Account {
            lamports: 1_000_000_000,
            data: bytes_of(fail_proof_data).to_vec(),
            owner: Pubkey::new_unique(),
            ..Account::default()
        },
    );
    let mut context = program_test.start_with_context().await;
    let rent = context.banks_client.get_rent().await.unwrap();

    let client = &mut context.banks_client;
    let payer = &context.payer;

    let context_state_account = Keypair::new();
    let context_state_authority = Keypair::new();

    let context_state_info = ContextStateInfo {
        context_state_account: &context_state_account.pubkey(),
        context_state_authority: &context_state_authority.pubkey(),
    };

    // try to create proof context state with an invalid proof
    let instructions = vec![
        system_instruction::create_account(
            &payer.pubkey(),
            &context_state_account.pubkey(),
            rent.minimum_balance(space),
            space as u64,
            &zk_elgamal_proof_program::id(),
        ),
        instruction_type.encode_verify_proof_from_account(
            Some(context_state_info),
            &fail_proof_account,
            0,
        ),
    ];
    let transaction = Transaction::new_signed_with_payer(
        &instructions,
        Some(&payer.pubkey()),
        &[payer, &context_state_account],
        client.get_latest_blockhash().await.unwrap(),
    );
    let err = client
        .process_transaction(transaction)
        .await
        .unwrap_err()
        .unwrap();
    assert_eq!(
        err,
        TransactionError::InstructionError(1, InstructionError::InvalidInstructionData)
    );

    // successfully create a proof context state
    let instructions = vec![
        system_instruction::create_account(
            &payer.pubkey(),
            &context_state_account.pubkey(),
            rent.minimum_balance(space),
            space as u64,
            &zk_elgamal_proof_program::id(),
        ),
        instruction_type.encode_verify_proof_from_account(
            Some(context_state_info),
            &success_proof_account,
            0,
        ),
    ];
    let transaction = Transaction::new_signed_with_payer(
        &instructions,
        Some(&payer.pubkey()),
        &[payer, &context_state_account],
        client.get_latest_blockhash().await.unwrap(),
    );
    client.process_transaction(transaction).await.unwrap();

    // try overwriting the context state
    let instructions = vec![instruction_type.encode_verify_proof_from_account(
        Some(context_state_info),
        &success_proof_account,
        0,
    )];
    let transaction = Transaction::new_signed_with_payer(
        &instructions,
        Some(&payer.pubkey()),
        &[payer],
        client.get_latest_blockhash().await.unwrap(),
    );
    let err = client
        .process_transaction(transaction)
        .await
        .unwrap_err()
        .unwrap();
    assert_eq!(
        err,
        TransactionError::InstructionError(0, InstructionError::AccountAlreadyInitialized)
    );

    // self-owned context state account
    let context_state_account_and_authority = Keypair::new();
    let context_state_info = ContextStateInfo {
        context_state_account: &context_state_account_and_authority.pubkey(),
        context_state_authority: &context_state_account_and_authority.pubkey(),
    };

    let instructions = vec![
        system_instruction::create_account(
            &payer.pubkey(),
            &context_state_account_and_authority.pubkey(),
            rent.minimum_balance(space),
            space as u64,
            &zk_elgamal_proof_program::id(),
        ),
        instruction_type.encode_verify_proof_from_account(
            Some(context_state_info),
            &success_proof_account,
            0,
        ),
    ];
    let transaction = Transaction::new_signed_with_payer(
        &instructions,
        Some(&payer.pubkey()),
        &[payer, &context_state_account_and_authority],
        client.get_latest_blockhash().await.unwrap(),
    );
    client.process_transaction(transaction).await.unwrap();
}

async fn test_close_context_state<T, U>(
    instruction_type: ProofInstruction,
    space: usize,
    success_proof_data: &T,
) where
    T: Pod + ZkProofData<U>,
    U: Pod,
{
    let mut program_test = ProgramTest::default();
    // the most expensive proof is the transfer-with-fee proof with 407_000 CUs
    program_test.set_compute_max_units(500_000);
    let mut context = program_test.start_with_context().await;
    let rent = context.banks_client.get_rent().await.unwrap();

    let client = &mut context.banks_client;
    let payer = &context.payer;

    let context_state_account = Keypair::new();
    let context_state_authority = Keypair::new();

    let context_state_info = ContextStateInfo {
        context_state_account: &context_state_account.pubkey(),
        context_state_authority: &context_state_authority.pubkey(),
    };

    let destination_account = Keypair::new();

    // create a proof context state
    let instructions = vec![
        system_instruction::create_account(
            &payer.pubkey(),
            &context_state_account.pubkey(),
            rent.minimum_balance(space),
            space as u64,
            &zk_elgamal_proof_program::id(),
        ),
        instruction_type.encode_verify_proof(Some(context_state_info), success_proof_data),
    ];
    let transaction = Transaction::new_signed_with_payer(
        &instructions.with_max_compute_unit_limit(),
        Some(&payer.pubkey()),
        &[payer, &context_state_account],
        client.get_latest_blockhash().await.unwrap(),
    );
    client.process_transaction(transaction).await.unwrap();

    // try to close context state with incorrect authority
    let incorrect_authority = Keypair::new();
    let instruction = close_context_state(
        ContextStateInfo {
            context_state_account: &context_state_account.pubkey(),
            context_state_authority: &incorrect_authority.pubkey(),
        },
        &destination_account.pubkey(),
    );
    let transaction = Transaction::new_signed_with_payer(
        &vec![instruction].with_max_compute_unit_limit(),
        Some(&payer.pubkey()),
        &[payer, &incorrect_authority],
        client.get_latest_blockhash().await.unwrap(),
    );
    let err = client
        .process_transaction(transaction)
        .await
        .unwrap_err()
        .unwrap();
    assert_eq!(
        err,
        TransactionError::InstructionError(0, InstructionError::InvalidAccountOwner)
    );

    // successfully close proof context state
    let instruction = close_context_state(
        ContextStateInfo {
            context_state_account: &context_state_account.pubkey(),
            context_state_authority: &context_state_authority.pubkey(),
        },
        &destination_account.pubkey(),
    );
    let transaction = Transaction::new_signed_with_payer(
        &vec![instruction.clone()].with_max_compute_unit_limit(),
        Some(&payer.pubkey()),
        &[payer, &context_state_authority],
        client.get_latest_blockhash().await.unwrap(),
    );
    client.process_transaction(transaction).await.unwrap();

    // create and close proof context in a single transaction
    let instructions = vec![
        system_instruction::create_account(
            &payer.pubkey(),
            &context_state_account.pubkey(),
            0_u64, // do not deposit rent
            space as u64,
            &zk_elgamal_proof_program::id(),
        ),
        instruction_type.encode_verify_proof(Some(context_state_info), success_proof_data),
        close_context_state(
            ContextStateInfo {
                context_state_account: &context_state_account.pubkey(),
                context_state_authority: &context_state_authority.pubkey(),
            },
            &destination_account.pubkey(),
        ),
    ];
    let transaction = Transaction::new_signed_with_payer(
        &instructions.with_max_compute_unit_limit(),
        Some(&payer.pubkey()),
        &[payer, &context_state_account, &context_state_authority],
        client.get_latest_blockhash().await.unwrap(),
    );
    client.process_transaction(transaction).await.unwrap();

    // close proof context state with owner as destination
    let instructions = vec![
        system_instruction::create_account(
            &payer.pubkey(),
            &context_state_account.pubkey(),
            0_u64,
            space as u64,
            &zk_elgamal_proof_program::id(),
        ),
        instruction_type.encode_verify_proof(Some(context_state_info), success_proof_data),
        close_context_state(
            ContextStateInfo {
                context_state_account: &context_state_account.pubkey(),
                context_state_authority: &context_state_authority.pubkey(),
            },
            &context_state_authority.pubkey(),
        ),
    ];
    let transaction = Transaction::new_signed_with_payer(
        &instructions.with_max_compute_unit_limit(),
        Some(&payer.pubkey()),
        &[payer, &context_state_account, &context_state_authority],
        client.get_latest_blockhash().await.unwrap(),
    );
    client.process_transaction(transaction).await.unwrap();

    // try close account with itself as destination
    let instructions = vec![
        system_instruction::create_account(
            &payer.pubkey(),
            &context_state_account.pubkey(),
            0_u64,
            space as u64,
            &zk_elgamal_proof_program::id(),
        ),
        instruction_type.encode_verify_proof(Some(context_state_info), success_proof_data),
        close_context_state(
            ContextStateInfo {
                context_state_account: &context_state_account.pubkey(),
                context_state_authority: &context_state_authority.pubkey(),
            },
            &context_state_account.pubkey(),
        ),
    ];
    let transaction = Transaction::new_signed_with_payer(
        &instructions.with_max_compute_unit_limit(),
        Some(&payer.pubkey()),
        &[payer, &context_state_account, &context_state_authority],
        client.get_latest_blockhash().await.unwrap(),
    );
    let err = client
        .process_transaction(transaction)
        .await
        .unwrap_err()
        .unwrap();
    assert_eq!(
        err,
        TransactionError::InstructionError(2, InstructionError::InvalidInstructionData)
    );

    // close self-owned proof context accounts
    let context_state_account_and_authority = Keypair::new();
    let context_state_info = ContextStateInfo {
        context_state_account: &context_state_account_and_authority.pubkey(),
        context_state_authority: &context_state_account_and_authority.pubkey(),
    };

    let instructions = vec![
        system_instruction::create_account(
            &payer.pubkey(),
            &context_state_account_and_authority.pubkey(),
            0_u64,
            space as u64,
            &zk_elgamal_proof_program::id(),
        ),
        instruction_type.encode_verify_proof(Some(context_state_info), success_proof_data),
        close_context_state(context_state_info, &context_state_account.pubkey()),
    ];
    let transaction = Transaction::new_signed_with_payer(
        &instructions.with_max_compute_unit_limit(),
        Some(&payer.pubkey()),
        &[payer, &context_state_account_and_authority],
        client.get_latest_blockhash().await.unwrap(),
    );
    client.process_transaction(transaction).await.unwrap();
}

// native programs consumes compute budget, some of ZK program consumes more than default
// budget (eg 200_000 CUs). To simplify tests, request Max cu for test transactions.
trait WithMaxComputeUnitLimit {
    fn with_max_compute_unit_limit(self) -> Self;
}

impl WithMaxComputeUnitLimit for Vec<gorbagana_instruction::Instruction> {
    fn with_max_compute_unit_limit(mut self) -> Self {
        self.push(
            gorbagana_compute_budget_interface::ComputeBudgetInstruction::set_compute_unit_limit(
                gorbagana_compute_budget::compute_budget_limits::MAX_COMPUTE_UNIT_LIMIT,
            ),
        );
        self
    }
}
