use {
    gorbagana_client::{
        nonblocking::tpu_client::TpuClient,
        rpc_config::RpcSendTransactionConfig,
        send_and_confirm_transactions_in_parallel::{
            send_and_confirm_transactions_in_parallel_blocking_v2, SendAndConfirmConfigV2,
        },
    },
    gorbagana_commitment_config::CommitmentConfig,
    gorbagana_keypair::Keypair,
    gorbagana_message::Message,
    gorbagana_native_token::sol_to_lamports,
    gorbagana_pubkey::Pubkey,
    gorbagana_rpc_client::rpc_client::RpcClient,
    gorbagana_signer::Signer,
    gorbagana_streamer::socket::SocketAddrSpace,
    gorbagana_system_interface::instruction as system_instruction,
    gorbagana_test_validator::TestValidator,
    std::sync::Arc,
};

const NUM_TRANSACTIONS: usize = 1000;

fn create_messages(from: Pubkey, to: Pubkey) -> (Vec<Message>, f64) {
    let mut messages = vec![];
    let mut sum = 0.0;
    for i in 1..NUM_TRANSACTIONS {
        let amount_to_transfer = i as f64;
        let ix = system_instruction::transfer(&from, &to, sol_to_lamports(amount_to_transfer));
        let message = Message::new(&[ix], Some(&from));
        messages.push(message);
        sum += amount_to_transfer;
    }
    (messages, sum)
}

#[test]
fn test_send_and_confirm_transactions_in_parallel_without_tpu_client() {
    gorbagana_logger::setup();

    let alice = Keypair::new();
    let test_validator =
        TestValidator::with_no_fees(alice.pubkey(), None, SocketAddrSpace::Unspecified);

    let bob_pubkey = gorbagana_pubkey::new_rand();
    let alice_pubkey = alice.pubkey();

    let rpc_client = Arc::new(RpcClient::new(test_validator.rpc_url()));

    assert_eq!(
        rpc_client.get_version().unwrap().gorbagana_core,
        gorbagana_version::semver!()
    );

    let original_alice_balance = rpc_client.get_balance(&alice.pubkey()).unwrap();
    let (messages, sum) = create_messages(alice_pubkey, bob_pubkey);

    let txs_errors = send_and_confirm_transactions_in_parallel_blocking_v2(
        rpc_client.clone(),
        None,
        &messages,
        &[&alice],
        SendAndConfirmConfigV2 {
            with_spinner: false,
            resign_txs_count: Some(5),
            rpc_send_transaction_config: RpcSendTransactionConfig {
                skip_preflight: false,
                preflight_commitment: Some(CommitmentConfig::confirmed().commitment),
                encoding: None,
                max_retries: None,
                min_context_slot: None,
            },
        },
    );
    assert!(txs_errors.is_ok());
    assert!(txs_errors.unwrap().iter().all(|x| x.is_none()));

    assert_eq!(
        rpc_client
            .get_balance_with_commitment(&bob_pubkey, CommitmentConfig::processed())
            .unwrap()
            .value,
        sol_to_lamports(sum)
    );
    assert_eq!(
        rpc_client
            .get_balance_with_commitment(&alice_pubkey, CommitmentConfig::processed())
            .unwrap()
            .value,
        original_alice_balance - sol_to_lamports(sum)
    );
}

#[test]
fn test_send_and_confirm_transactions_in_parallel_with_tpu_client() {
    gorbagana_logger::setup();

    let alice = Keypair::new();
    let test_validator =
        TestValidator::with_no_fees(alice.pubkey(), None, SocketAddrSpace::Unspecified);

    let bob_pubkey = gorbagana_pubkey::new_rand();
    let alice_pubkey = alice.pubkey();

    let rpc_client = Arc::new(RpcClient::new(test_validator.rpc_url()));

    assert_eq!(
        rpc_client.get_version().unwrap().gorbagana_core,
        gorbagana_version::semver!()
    );

    let original_alice_balance = rpc_client.get_balance(&alice.pubkey()).unwrap();
    let (messages, sum) = create_messages(alice_pubkey, bob_pubkey);
    let ws_url = test_validator.rpc_pubsub_url();
    let tpu_client_fut = TpuClient::new(
        "temp",
        rpc_client.get_inner_client().clone(),
        ws_url.as_str(),
        gorbagana_client::tpu_client::TpuClientConfig::default(),
    );
    let tpu_client = rpc_client.runtime().block_on(tpu_client_fut).unwrap();

    let txs_errors = send_and_confirm_transactions_in_parallel_blocking_v2(
        rpc_client.clone(),
        Some(tpu_client),
        &messages,
        &[&alice],
        SendAndConfirmConfigV2 {
            with_spinner: false,
            resign_txs_count: Some(5),
            rpc_send_transaction_config: RpcSendTransactionConfig {
                skip_preflight: false,
                preflight_commitment: Some(CommitmentConfig::confirmed().commitment),
                encoding: None,
                max_retries: None,
                min_context_slot: None,
            },
        },
    );
    assert!(txs_errors.is_ok());
    assert!(txs_errors.unwrap().iter().all(|x| x.is_none()));

    assert_eq!(
        rpc_client
            .get_balance_with_commitment(&bob_pubkey, CommitmentConfig::processed())
            .unwrap()
            .value,
        sol_to_lamports(sum)
    );
    assert_eq!(
        rpc_client
            .get_balance_with_commitment(&alice_pubkey, CommitmentConfig::processed())
            .unwrap()
            .value,
        original_alice_balance - sol_to_lamports(sum)
    );
}
