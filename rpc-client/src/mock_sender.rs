//! A nonblocking [`RpcSender`] used for unit testing [`RpcClient`](crate::rpc_client::RpcClient).

use {
    crate::rpc_sender::*,
    async_trait::async_trait,
    base64::{prelude::BASE64_STANDARD, Engine},
    serde_json::{json, Number, Value},
    gorbagana_account_decoder_client_types::{UiAccount, UiAccountData, UiAccountEncoding},
    gorbagana_clock::{Slot, UnixTimestamp},
    gorbagana_epoch_info::EpochInfo,
    gorbagana_epoch_schedule::EpochSchedule,
    gorbagana_instruction::{error::InstructionError, TRANSACTION_LEVEL_STACK_HEIGHT},
    gorbagana_message::MessageHeader,
    gorbagana_pubkey::Pubkey,
    gorbagana_rpc_client_api::{
        client_error::Result,
        config::RpcBlockProductionConfig,
        request::RpcRequest,
        response::{
            Response, RpcAccountBalance, RpcBlockProduction, RpcBlockProductionRange, RpcBlockhash,
            RpcConfirmedTransactionStatusWithSignature, RpcContactInfo, RpcIdentity,
            RpcInflationGovernor, RpcInflationRate, RpcInflationReward, RpcKeyedAccount,
            RpcPerfSample, RpcPrioritizationFee, RpcResponseContext, RpcSimulateTransactionResult,
            RpcSnapshotSlotInfo, RpcSupply, RpcVersionInfo, RpcVoteAccountInfo,
            RpcVoteAccountStatus,
        },
    },
    gorbagana_signature::Signature,
    gorbagana_transaction::{versioned::TransactionVersion, Transaction},
    gorbagana_transaction_error::{TransactionError, TransactionResult},
    gorbagana_transaction_status_client_types::{
        option_serializer::OptionSerializer, EncodedConfirmedBlock,
        EncodedConfirmedTransactionWithStatusMeta, EncodedTransaction,
        EncodedTransactionWithStatusMeta, Rewards, TransactionBinaryEncoding,
        TransactionConfirmationStatus, TransactionStatus, UiCompiledInstruction, UiMessage,
        UiRawMessage, UiTransaction, UiTransactionStatusMeta,
    },
    gorbagana_version::Version,
    std::{
        collections::{HashMap, VecDeque},
        net::SocketAddr,
        str::FromStr,
        sync::RwLock,
    },
};

pub const PUBKEY: &str = "7RoSF9fUmdphVCpabEoefH81WwrW7orsWonXWqTXkKV8";

pub type Mocks = HashMap<RpcRequest, Value>;

impl From<Mocks> for MocksMap {
    fn from(mocks: Mocks) -> Self {
        let mut map = HashMap::new();
        for (key, value) in mocks {
            map.insert(key, [value].into());
        }
        MocksMap(map)
    }
}

#[derive(Default, Clone)]
pub struct MocksMap(pub HashMap<RpcRequest, VecDeque<Value>>);

impl FromIterator<(RpcRequest, Value)> for MocksMap {
    fn from_iter<T: IntoIterator<Item = (RpcRequest, Value)>>(iter: T) -> Self {
        let mut map = MocksMap::default();
        for (request, value) in iter {
            map.insert(request, value);
        }
        map
    }
}

impl MocksMap {
    pub fn insert(&mut self, request: RpcRequest, value: Value) {
        let queue = self.0.entry(request).or_default();
        queue.push_back(value)
    }

    pub fn pop_front_with_request(&mut self, request: &RpcRequest) -> Option<Value> {
        self.0.get_mut(request).and_then(|queue| queue.pop_front())
    }
}

pub struct MockSender {
    mocks: RwLock<MocksMap>,
    url: String,
}

/// An [`RpcSender`] used for unit testing [`RpcClient`](crate::rpc_client::RpcClient).
///
/// This is primarily for internal use.
///
/// Unless directed otherwise, it will generally return a reasonable default
/// response, at least for [`RpcRequest`] values for which responses have been
/// implemented.
///
/// The behavior can be customized in two ways:
///
/// 1) The `url` constructor argument is not actually a URL, but a simple string
///    directive that changes `MockSender`s behavior in specific scenarios.
///
///    If `url` is "fails" then any call to `send` will return `Ok(Value::Null)`.
///
///    It is customary to set the `url` to "succeeds" for mocks that should
///    return successfully, though this value is not actually interpreted.
///
///    Other possible values of `url` are specific to different `RpcRequest`
///    values. Read the implementation for specifics.
///
/// 2) Custom responses can be configured by providing [`Mocks`] to the
///    [`MockSender::new_with_mocks`] constructor. This type is a [`HashMap`]
///    from [`RpcRequest`] to a JSON [`Value`] response, Any entries in this map
///    override the default behavior for the given request.
impl MockSender {
    pub fn new<U: ToString>(url: U) -> Self {
        Self::new_with_mocks(url, Mocks::default())
    }

    pub fn new_with_mocks<U: ToString>(url: U, mocks: Mocks) -> Self {
        Self {
            url: url.to_string(),
            mocks: RwLock::new(MocksMap::from(mocks)),
        }
    }

    pub fn new_with_mocks_map<U: ToString>(url: U, mocks: MocksMap) -> Self {
        Self {
            url: url.to_string(),
            mocks: RwLock::new(mocks),
        }
    }
}

#[async_trait]
impl RpcSender for MockSender {
    fn get_transport_stats(&self) -> RpcTransportStats {
        RpcTransportStats::default()
    }

    async fn send(
        &self,
        request: RpcRequest,
        params: serde_json::Value,
    ) -> Result<serde_json::Value> {
        if let Some(value) = self.mocks.write().unwrap().pop_front_with_request(&request) {
            return Ok(value);
        }
        if self.url == "fails" {
            return Ok(Value::Null);
        }

        let method = &request.build_request_json(42, params.clone())["method"];

        let val = match method.as_str().unwrap() {
            "getAccountInfo" => serde_json::to_value(Response {
                context: RpcResponseContext { slot: 1, api_version: None },
                value: Value::Null,
            })?,
            "getBalance" => serde_json::to_value(Response {
                context: RpcResponseContext { slot: 1, api_version: None },
                value: Value::Number(Number::from(50)),
            })?,
            "getEpochInfo" => serde_json::to_value(EpochInfo {
                epoch: 1,
                slot_index: 2,
                slots_in_epoch: 32,
                absolute_slot: 34,
                block_height: 34,
                transaction_count: Some(123),
            })?,
            "getSignatureStatuses" => {
                let status: TransactionResult<()> = if self.url == "account_in_use" {
                    Err(TransactionError::AccountInUse)
                } else if self.url == "instruction_error" {
                    Err(TransactionError::InstructionError(
                        0,
                        InstructionError::UninitializedAccount,
                    ))
                } else {
                    Ok(())
                };
                let status = if self.url == "sig_not_found" {
                    None
                } else {
                    let err = status.clone().err();
                    Some(TransactionStatus {
                        status,
                        slot: 1,
                        confirmations: None,
                        err,
                        confirmation_status: Some(TransactionConfirmationStatus::Finalized),
                    })
                };
                let statuses: Vec<Option<TransactionStatus>> = params.as_array().unwrap()[0]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|_| status.clone())
                    .collect();
                serde_json::to_value(Response {
                    context: RpcResponseContext { slot: 1, api_version: None },
                    value: statuses,
                })?
            }
            "getTransaction" => serde_json::to_value(EncodedConfirmedTransactionWithStatusMeta {
                slot: 2,
                transaction: EncodedTransactionWithStatusMeta {
                    version: Some(TransactionVersion::LEGACY),
                    transaction: EncodedTransaction::Json(
                        UiTransaction {
                            signatures: vec!["3AsdoALgZFuq2oUVWrDYhg2pNeaLJKPLf8hU2mQ6U8qJxeJ6hsrPVpMn9ma39DtfYCrDQSvngWRP8NnTpEhezJpE".to_string()],
                            message: UiMessage::Raw(
                                UiRawMessage {
                                    header: MessageHeader {
                                        num_required_signatures: 1,
                                        num_readonly_signed_accounts: 0,
                                        num_readonly_unsigned_accounts: 1,
                                    },
                                    account_keys: vec![
                                        "C6eBmAXKg6JhJWkajGa5YRGUfG4YKXwbxF5Ufv7PtExZ".to_string(),
                                        "2Gd5eoR5J4BV89uXbtunpbNhjmw3wa1NbRHxTHzDzZLX".to_string(),
                                        "11111111111111111111111111111111".to_string(),
                                    ],
                                    recent_blockhash: "D37n3BSG71oUWcWjbZ37jZP7UfsxG2QMKeuALJ1PYvM6".to_string(),
                                    instructions: vec![UiCompiledInstruction {
                                        program_id_index: 2,
                                        accounts: vec![0, 1],
                                        data: "3Bxs49DitAvXtoDR".to_string(),
                                        stack_height: Some(TRANSACTION_LEVEL_STACK_HEIGHT as u32),
                                    }],
                                    address_table_lookups: None,
                                })
                        }),
                    meta: Some(UiTransactionStatusMeta {
                            err: None,
                            status: Ok(()),
                            fee: 0,
                            pre_balances: vec![499999999999999950, 50, 1],
                            post_balances: vec![499999999999999950, 50, 1],
                            inner_instructions: OptionSerializer::None,
                            log_messages: OptionSerializer::None,
                            pre_token_balances: OptionSerializer::None,
                            post_token_balances: OptionSerializer::None,
                            rewards: OptionSerializer::None,
                            loaded_addresses: OptionSerializer::Skip,
                            return_data: OptionSerializer::Skip,
                            compute_units_consumed: OptionSerializer::Skip,
                            cost_units: OptionSerializer::Skip,
                        }),
                },
                block_time: Some(1628633791),
            })?,
            "getTransactionCount" => json![1234],
            "getSlot" => json![0],
            "getMaxShredInsertSlot" => json![0],
            "requestAirdrop" => Value::String(Signature::from([8; 64]).to_string()),
            "getHighestSnapshotSlot" => json!(RpcSnapshotSlotInfo {
                full: 100,
                incremental: Some(110),
            }),
            "getBlockHeight" => Value::Number(Number::from(1234)),
            "getSlotLeaders" => json!([PUBKEY]),
            "getBlockProduction" => {
                if params.is_null() {
                    json!(Response {
                        context: RpcResponseContext { slot: 1, api_version: None },
                        value: RpcBlockProduction {
                            by_identity: HashMap::new(),
                            range: RpcBlockProductionRange {
                                first_slot: 1,
                                last_slot: 2,
                            },
                        },
                    })
                } else {
                    let config: Vec<RpcBlockProductionConfig> =
                        serde_json::from_value(params).unwrap();
                    let config = config[0].clone();
                    let mut by_identity = HashMap::new();
                    by_identity.insert(config.identity.unwrap(), (1, 123));
                    let config_range = config.range.unwrap_or_default();

                    json!(Response {
                        context: RpcResponseContext { slot: 1, api_version: None },
                        value: RpcBlockProduction {
                            by_identity,
                            range: RpcBlockProductionRange {
                                first_slot: config_range.first_slot,
                                last_slot: {
                                    config_range.last_slot.unwrap_or(2)
                                },
                            },
                        },
                    })
                }
            }
            "getStakeMinimumDelegation" => json!(Response {
                context: RpcResponseContext { slot: 1, api_version: None },
                value: 123_456_789,
            }),
            "getSupply" => json!(Response {
                context: RpcResponseContext { slot: 1, api_version: None },
                value: RpcSupply {
                    total: 100000000,
                    circulating: 50000,
                    non_circulating: 20000,
                    non_circulating_accounts: vec![PUBKEY.to_string()],
                },
            }),
            "getLargestAccounts" => {
                let rpc_account_balance = RpcAccountBalance {
                    address: PUBKEY.to_string(),
                    lamports: 10000,
                };

                json!(Response {
                    context: RpcResponseContext { slot: 1, api_version: None },
                    value: vec![rpc_account_balance],
                })
            }
            "getVoteAccounts" => {
                json!(RpcVoteAccountStatus {
                    current: vec![],
                    delinquent: vec![RpcVoteAccountInfo {
                        vote_pubkey: PUBKEY.to_string(),
                        node_pubkey: PUBKEY.to_string(),
                        activated_stake: 0,
                        commission: 0,
                        epoch_vote_account: false,
                        epoch_credits: vec![],
                        last_vote: 0,
                        root_slot: Slot::default(),
                    }],
                })
            }
            "sendTransaction" => {
                let signature = if self.url == "malicious" {
                    Signature::from([8; 64]).to_string()
                } else {
                    let tx_str = params.as_array().unwrap()[0].as_str().unwrap().to_string();
                    let data = BASE64_STANDARD.decode(tx_str).unwrap();
                    let tx: Transaction = bincode::deserialize(&data).unwrap();
                    tx.signatures[0].to_string()
                };
                Value::String(signature)
            }
            "simulateTransaction" => serde_json::to_value(Response {
                context: RpcResponseContext { slot: 1, api_version: None },
                value: RpcSimulateTransactionResult {
                    err: None,
                    logs: None,
                    accounts: None,
                    units_consumed: None,
                    loaded_accounts_data_size: None,
                    return_data: None,
                    inner_instructions: None,
                    replacement_blockhash: None
                },
            })?,
            "getMinimumBalanceForRentExemption" => json![20],
            "getVersion" => {
                let version = Version::default();
                json!(RpcVersionInfo {
                    gorbagana_core: version.to_string(),
                    feature_set: Some(version.feature_set),
                })
            }
            "getLatestBlockhash" => serde_json::to_value(Response {
                context: RpcResponseContext { slot: 1, api_version: None },
                value: RpcBlockhash {
                    blockhash: PUBKEY.to_string(),
                    last_valid_block_height: 1234,
                },
            })?,
            "getFeeForMessage" => serde_json::to_value(Response {
                context: RpcResponseContext { slot: 1, api_version: None },
                value: json!(Some(0)),
            })?,
            "getClusterNodes" => serde_json::to_value(vec![RpcContactInfo {
                pubkey: PUBKEY.to_string(),
                gossip: Some(SocketAddr::from(([10, 239, 6, 48], 8899))),
                tvu: Some(SocketAddr::from(([10, 239, 6, 48], 8865))),
                tpu: Some(SocketAddr::from(([10, 239, 6, 48], 8856))),
                tpu_quic: Some(SocketAddr::from(([10, 239, 6, 48], 8862))),
                tpu_forwards: Some(SocketAddr::from(([10, 239, 6, 48], 8857))),
                tpu_forwards_quic: Some(SocketAddr::from(([10, 239, 6, 48], 8863))),
                tpu_vote: Some(SocketAddr::from(([10, 239, 6, 48], 8870))),
                serve_repair: Some(SocketAddr::from(([10, 239, 6, 48], 8880))),
                rpc: Some(SocketAddr::from(([10, 239, 6, 48], 8899))),
                pubsub: Some(SocketAddr::from(([10, 239, 6, 48], 8900))),
                version: Some("1.0.0 c375ce1f".to_string()),
                feature_set: None,
                shred_version: None,
            }])?,
            "getBlock" => serde_json::to_value(EncodedConfirmedBlock {
                previous_blockhash: "mfcyqEXB3DnHXki6KjjmZck6YjmZLvpAByy2fj4nh6B".to_string(),
                blockhash: "3Eq21vXNB5s86c62bVuUfTeaMif1N2kUqRPBmGRJhyTA".to_string(),
                parent_slot: 429,
                transactions: vec![EncodedTransactionWithStatusMeta {
                    transaction: EncodedTransaction::Binary(
                        "ju9xZWuDBX4pRxX2oZkTjxU5jB4SSTgEGhX8bQ8PURNzyzqKMPPpNvWihx8zUe\
                                 FfrbVNoAaEsNKZvGzAnTDy5bhNT9kt6KFCTBixpvrLCzg4M5UdFUQYrn1gdgjX\
                                 pLHxcaShD81xBNaFDgnA2nkkdHnKtZt4hVSfKAmw3VRZbjrZ7L2fKZBx21CwsG\
                                 hD6onjM2M3qZW5C8J6d1pj41MxKmZgPBSha3MyKkNLkAGFASK"
                            .to_string(),
                        TransactionBinaryEncoding::Base58,
                    ),
                    meta: None,
                    version: Some(TransactionVersion::LEGACY),
                }],
                rewards: Rewards::new(),
                num_partitions: None,
                block_time: None,
                block_height: Some(428),
            })?,
            "getBlocks" => serde_json::to_value(vec![1, 2, 3])?,
            "getBlocksWithLimit" => serde_json::to_value(vec![1, 2, 3])?,
            "getSignaturesForAddress" => {
                serde_json::to_value(vec![RpcConfirmedTransactionStatusWithSignature {
                    signature: crate::mock_sender_for_cli::SIGNATURE.to_string(),
                    slot: 123,
                    err: None,
                    memo: None,
                    block_time: None,
                    confirmation_status: Some(TransactionConfirmationStatus::Finalized),
                }])?
            }
            "getBlockTime" => serde_json::to_value(UnixTimestamp::default())?,
            "getEpochSchedule" => serde_json::to_value(EpochSchedule::default())?,
            "getRecentPerformanceSamples" => serde_json::to_value(vec![RpcPerfSample {
                slot: 347873,
                num_transactions: 125,
                num_non_vote_transactions: Some(1),
                num_slots: 123,
                sample_period_secs: 60,
            }])?,
            "getRecentPrioritizationFees" => serde_json::to_value(vec![RpcPrioritizationFee {
                slot: 123_456_789,
                prioritization_fee: 10_000,
            }])?,
            "getIdentity" => serde_json::to_value(RpcIdentity {
                identity: PUBKEY.to_string(),
            })?,
            "getInflationGovernor" => serde_json::to_value(
                RpcInflationGovernor {
                    initial: 0.08,
                    terminal: 0.015,
                    taper: 0.15,
                    foundation: 0.05,
                    foundation_term: 7.0,
                })?,
            "getInflationRate" => serde_json::to_value(
                RpcInflationRate {
                    total: 0.08,
                    validator: 0.076,
                    foundation: 0.004,
                    epoch: 0,
                })?,
            "getInflationReward" => serde_json::to_value(vec![
                Some(RpcInflationReward {
                    epoch: 2,
                    effective_slot: 224,
                    amount: 2500,
                    post_balance: 499999442500,
                    commission: None,
                })])?,
            "minimumLedgerSlot" => json![123],
            "getMaxRetransmitSlot" => json![123],
            "getMultipleAccounts" => serde_json::to_value(Response {
                context: RpcResponseContext { slot: 1, api_version: None },
                value: vec![Value::Null, Value::Null]
            })?,
            "getProgramAccounts" => {
                let pubkey = Pubkey::from_str(PUBKEY).unwrap();
                serde_json::to_value(vec![
                    RpcKeyedAccount {
                        pubkey: PUBKEY.to_string(),
                        account: mock_encoded_account(&pubkey)
                    }
                ])?
            },
            _ => Value::Null,
        };
        Ok(val)
    }

    fn url(&self) -> String {
        format!("MockSender: {}", self.url)
    }
}

pub(crate) fn mock_encoded_account(pubkey: &Pubkey) -> UiAccount {
    UiAccount {
        lamports: 1_000_000,
        data: UiAccountData::Binary("".to_string(), UiAccountEncoding::Base64),
        owner: pubkey.to_string(),
        executable: false,
        rent_epoch: 0,
        space: Some(0),
    }
}

#[cfg(test)]
mod tests {
    use {super::*, gorbagana_account::Account, gorbagana_account_decoder::encode_ui_account};

    #[test]
    fn test_mock_encoded_account() {
        let pubkey = Pubkey::from_str(PUBKEY).unwrap();
        let account = Account {
            lamports: 1_000_000,
            data: vec![],
            owner: pubkey,
            executable: false,
            rent_epoch: 0,
        };
        let expected = encode_ui_account(&pubkey, &account, UiAccountEncoding::Base64, None, None);
        assert_eq!(expected, mock_encoded_account(&pubkey));
    }
}
