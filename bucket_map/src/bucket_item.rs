use {crate::RefCount, gorbagana_pubkey::Pubkey};

#[derive(Debug, Default, Clone)]
pub struct BucketItem<T> {
    pub pubkey: Pubkey,
    pub ref_count: RefCount,
    pub slot_list: Vec<T>,
}
