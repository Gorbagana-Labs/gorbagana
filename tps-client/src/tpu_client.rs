use {
    crate::{TpsClient, TpsClientError, TpsClientResult},
    gorbagana_account::Account,
    gorbagana_commitment_config::CommitmentConfig,
    gorbagana_connection_cache::connection_cache::{
        ConnectionManager, ConnectionPool, NewConnectionConfig,
    },
    gorbagana_epoch_info::EpochInfo,
    gorbagana_hash::Hash,
    gorbagana_message::Message,
    gorbagana_pubkey::Pubkey,
    gorbagana_rpc_client_api::config::RpcBlockConfig,
    gorbagana_signature::Signature,
    gorbagana_tpu_client::tpu_client::TpuClient,
    gorbagana_transaction::Transaction,
    gorbagana_transaction_error::TransactionResult as Result,
    gorbagana_transaction_status::UiConfirmedBlock,
};

impl<P, M, C> TpsClient for TpuClient<P, M, C>
where
    P: ConnectionPool<NewConnectionConfig = C>,
    M: ConnectionManager<ConnectionPool = P, NewConnectionConfig = C>,
    C: NewConnectionConfig,
{
    fn send_transaction(&self, transaction: Transaction) -> TpsClientResult<Signature> {
        let signature = transaction.signatures[0];
        self.try_send_transaction(&transaction)?;
        Ok(signature)
    }
    fn send_batch(&self, transactions: Vec<Transaction>) -> TpsClientResult<()> {
        self.try_send_transaction_batch(&transactions)?;
        Ok(())
    }
    fn get_latest_blockhash(&self) -> TpsClientResult<Hash> {
        self.rpc_client()
            .get_latest_blockhash()
            .map_err(|err| err.into())
    }

    fn get_latest_blockhash_with_commitment(
        &self,
        commitment_config: CommitmentConfig,
    ) -> TpsClientResult<(Hash, u64)> {
        self.rpc_client()
            .get_latest_blockhash_with_commitment(commitment_config)
            .map_err(|err| err.into())
    }

    fn get_new_latest_blockhash(&self, blockhash: &Hash) -> TpsClientResult<Hash> {
        self.rpc_client()
            .get_new_latest_blockhash(blockhash)
            .map_err(|err| err.into())
    }

    fn get_signature_status(&self, signature: &Signature) -> TpsClientResult<Option<Result<()>>> {
        self.rpc_client()
            .get_signature_status(signature)
            .map_err(|err| err.into())
    }

    fn get_transaction_count(&self) -> TpsClientResult<u64> {
        self.rpc_client()
            .get_transaction_count()
            .map_err(|err| err.into())
    }

    fn get_transaction_count_with_commitment(
        &self,
        commitment_config: CommitmentConfig,
    ) -> TpsClientResult<u64> {
        self.rpc_client()
            .get_transaction_count_with_commitment(commitment_config)
            .map_err(|err| err.into())
    }

    fn get_epoch_info(&self) -> TpsClientResult<EpochInfo> {
        self.rpc_client().get_epoch_info().map_err(|err| err.into())
    }

    fn get_balance(&self, pubkey: &Pubkey) -> TpsClientResult<u64> {
        self.rpc_client()
            .get_balance(pubkey)
            .map_err(|err| err.into())
    }

    fn get_balance_with_commitment(
        &self,
        pubkey: &Pubkey,
        commitment_config: CommitmentConfig,
    ) -> TpsClientResult<u64> {
        self.rpc_client()
            .get_balance_with_commitment(pubkey, commitment_config)
            .map(|res| res.value)
            .map_err(|err| err.into())
    }

    fn get_fee_for_message(&self, message: &Message) -> TpsClientResult<u64> {
        self.rpc_client()
            .get_fee_for_message(message)
            .map_err(|err| err.into())
    }

    fn get_minimum_balance_for_rent_exemption(&self, data_len: usize) -> TpsClientResult<u64> {
        self.rpc_client()
            .get_minimum_balance_for_rent_exemption(data_len)
            .map_err(|err| err.into())
    }

    fn addr(&self) -> String {
        self.rpc_client().url()
    }

    fn request_airdrop_with_blockhash(
        &self,
        pubkey: &Pubkey,
        lamports: u64,
        recent_blockhash: &Hash,
    ) -> TpsClientResult<Signature> {
        self.rpc_client()
            .request_airdrop_with_blockhash(pubkey, lamports, recent_blockhash)
            .map_err(|err| err.into())
    }

    fn get_account(&self, pubkey: &Pubkey) -> TpsClientResult<Account> {
        self.rpc_client()
            .get_account(pubkey)
            .map_err(|err| err.into())
    }

    fn get_account_with_commitment(
        &self,
        pubkey: &Pubkey,
        commitment_config: CommitmentConfig,
    ) -> TpsClientResult<Account> {
        self.rpc_client()
            .get_account_with_commitment(pubkey, commitment_config)
            .map(|res| res.value)
            .map_err(|err| err.into())
            .and_then(|account| {
                account.ok_or_else(|| {
                    TpsClientError::Custom(format!("AccountNotFound: pubkey={pubkey}"))
                })
            })
    }

    fn get_multiple_accounts(&self, pubkeys: &[Pubkey]) -> TpsClientResult<Vec<Option<Account>>> {
        self.rpc_client()
            .get_multiple_accounts(pubkeys)
            .map_err(|err| err.into())
    }

    fn get_slot_with_commitment(
        &self,
        commitment_config: CommitmentConfig,
    ) -> TpsClientResult<u64> {
        self.rpc_client()
            .get_slot_with_commitment(commitment_config)
            .map_err(|err| err.into())
    }

    fn get_blocks_with_commitment(
        &self,
        start_slot: u64,
        end_slot: Option<u64>,
        commitment_config: CommitmentConfig,
    ) -> TpsClientResult<Vec<u64>> {
        self.rpc_client()
            .get_blocks_with_commitment(start_slot, end_slot, commitment_config)
            .map_err(|err| err.into())
    }

    fn get_block_with_config(
        &self,
        slot: u64,
        rpc_block_config: RpcBlockConfig,
    ) -> TpsClientResult<UiConfirmedBlock> {
        self.rpc_client()
            .get_block_with_config(slot, rpc_block_config)
            .map_err(|err| err.into())
    }
}
