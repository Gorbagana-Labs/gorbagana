use {
    gorbagana_hash::Hash,
    gorbagana_keypair::Keypair,
    gorbagana_pubkey::Pubkey,
    gorbagana_system_transaction::transfer,
    gorbagana_transaction::Transaction,
    std::{io::Error, net::SocketAddr},
};

pub fn request_airdrop_transaction(
    _faucet_addr: &SocketAddr,
    _id: &Pubkey,
    lamports: u64,
    _blockhash: Hash,
) -> Result<Transaction, Error> {
    if lamports == 0 {
        Err(Error::other("Airdrop failed"))
    } else {
        let key = Keypair::new();
        let to = gorbagana_pubkey::new_rand();
        let blockhash = Hash::default();
        let tx = transfer(&key, &to, lamports, blockhash);
        Ok(tx)
    }
}
