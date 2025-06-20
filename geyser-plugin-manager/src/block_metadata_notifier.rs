use {
    crate::{
        block_metadata_notifier_interface::BlockMetadataNotifier,
        geyser_plugin_manager::GeyserPluginManager,
    },
    agave_geyser_plugin_interface::geyser_plugin_interface::{
        ReplicaBlockInfoV4, ReplicaBlockInfoVersions,
    },
    log::*,
    gorbagana_clock::UnixTimestamp,
    gorbagana_measure::measure::Measure,
    gorbagana_metrics::*,
    gorbagana_runtime::bank::KeyedRewardsAndNumPartitions,
    gorbagana_transaction_status::{Reward, RewardsAndNumPartitions},
    std::sync::{Arc, RwLock},
};

pub(crate) struct BlockMetadataNotifierImpl {
    plugin_manager: Arc<RwLock<GeyserPluginManager>>,
}

impl BlockMetadataNotifier for BlockMetadataNotifierImpl {
    /// Notify the block metadata
    fn notify_block_metadata(
        &self,
        parent_slot: u64,
        parent_blockhash: &str,
        slot: u64,
        blockhash: &str,
        rewards: &KeyedRewardsAndNumPartitions,
        block_time: Option<UnixTimestamp>,
        block_height: Option<u64>,
        executed_transaction_count: u64,
        entry_count: u64,
    ) {
        let plugin_manager = self.plugin_manager.read().unwrap();
        if plugin_manager.plugins.is_empty() {
            return;
        }

        let rewards = Self::build_rewards(rewards);
        let block_info = Self::build_replica_block_info(
            parent_slot,
            parent_blockhash,
            slot,
            blockhash,
            &rewards,
            block_time,
            block_height,
            executed_transaction_count,
            entry_count,
        );

        for plugin in plugin_manager.plugins.iter() {
            let mut measure = Measure::start("geyser-plugin-update-slot");
            let block_info = ReplicaBlockInfoVersions::V0_0_4(&block_info);
            match plugin.notify_block_metadata(block_info) {
                Err(err) => {
                    error!(
                        "Failed to update block metadata at slot {}, error: {} to plugin {}",
                        slot,
                        err,
                        plugin.name()
                    )
                }
                Ok(_) => {
                    trace!(
                        "Successfully updated block metadata at slot {} to plugin {}",
                        slot,
                        plugin.name()
                    );
                }
            }
            measure.stop();
            inc_new_counter_debug!(
                "geyser-plugin-update-block-metadata-us",
                measure.as_us() as usize,
                1000,
                1000
            );
        }
    }
}

impl BlockMetadataNotifierImpl {
    fn build_rewards(rewards: &KeyedRewardsAndNumPartitions) -> RewardsAndNumPartitions {
        RewardsAndNumPartitions {
            rewards: rewards
                .keyed_rewards
                .iter()
                .map(|(pubkey, reward)| Reward {
                    pubkey: pubkey.to_string(),
                    lamports: reward.lamports,
                    post_balance: reward.post_balance,
                    reward_type: Some(reward.reward_type),
                    commission: reward.commission,
                })
                .collect(),
            num_partitions: rewards.num_partitions,
        }
    }

    fn build_replica_block_info<'a>(
        parent_slot: u64,
        parent_blockhash: &'a str,
        slot: u64,
        blockhash: &'a str,
        rewards: &'a RewardsAndNumPartitions,
        block_time: Option<UnixTimestamp>,
        block_height: Option<u64>,
        executed_transaction_count: u64,
        entry_count: u64,
    ) -> ReplicaBlockInfoV4<'a> {
        ReplicaBlockInfoV4 {
            parent_slot,
            parent_blockhash,
            slot,
            blockhash,
            rewards,
            block_time,
            block_height,
            executed_transaction_count,
            entry_count,
        }
    }

    pub fn new(plugin_manager: Arc<RwLock<GeyserPluginManager>>) -> Self {
        Self { plugin_manager }
    }
}
