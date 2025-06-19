#[cfg(test)]
use gorbagana_perf::packet::PacketRef;
use {
    super::immutable_deserialized_packet::{DeserializedPacketError, ImmutableDeserializedPacket},
    gorbagana_bincode::limited_deserialize,
    gorbagana_clock::{Slot, UnixTimestamp},
    gorbagana_hash::Hash,
    gorbagana_packet::PACKET_DATA_SIZE,
    gorbagana_pubkey::Pubkey,
    gorbagana_vote_program::vote_instruction::VoteInstruction,
    std::sync::Arc,
};

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum VoteSource {
    Gossip,
    Tpu,
}

/// Holds deserialized vote messages as well as their source, and slot
#[derive(Debug, Clone)]
pub struct LatestValidatorVotePacket {
    vote_source: VoteSource,
    vote_pubkey: Pubkey,
    vote: Option<Arc<ImmutableDeserializedPacket>>,
    slot: Slot,
    hash: Hash,
    timestamp: Option<UnixTimestamp>,
}

impl LatestValidatorVotePacket {
    pub fn new_from_immutable(
        vote: Arc<ImmutableDeserializedPacket>,
        vote_source: VoteSource,
        deprecate_legacy_vote_ixs: bool,
    ) -> Result<Self, DeserializedPacketError> {
        let message = vote.transaction().get_message();
        let (_, instruction) = message
            .program_instructions_iter()
            .next()
            .ok_or(DeserializedPacketError::VoteTransactionError)?;

        let instruction_filter = |ix: &VoteInstruction| {
            if deprecate_legacy_vote_ixs {
                matches!(
                    ix,
                    VoteInstruction::TowerSync(_) | VoteInstruction::TowerSyncSwitch(_, _),
                )
            } else {
                ix.is_single_vote_state_update()
            }
        };

        match limited_deserialize::<VoteInstruction>(&instruction.data, PACKET_DATA_SIZE as u64) {
            Ok(vote_state_update_instruction)
                if instruction_filter(&vote_state_update_instruction) =>
            {
                let vote_account_index = instruction
                    .accounts
                    .first()
                    .copied()
                    .ok_or(DeserializedPacketError::VoteTransactionError)?;
                let vote_pubkey = message
                    .message
                    .static_account_keys()
                    .get(vote_account_index as usize)
                    .copied()
                    .ok_or(DeserializedPacketError::VoteTransactionError)?;
                let slot = vote_state_update_instruction.last_voted_slot().unwrap_or(0);
                let hash = vote_state_update_instruction.hash();
                let timestamp = vote_state_update_instruction.timestamp();

                Ok(Self {
                    vote: Some(vote),
                    slot,
                    hash,
                    vote_pubkey,
                    vote_source,
                    timestamp,
                })
            }
            _ => Err(DeserializedPacketError::VoteTransactionError),
        }
    }

    #[cfg(test)]
    pub fn new(
        packet: PacketRef,
        vote_source: VoteSource,
        deprecate_legacy_vote_ixs: bool,
    ) -> Result<Self, DeserializedPacketError> {
        if !packet.meta().is_simple_vote_tx() {
            return Err(DeserializedPacketError::VoteTransactionError);
        }

        let vote = Arc::new(ImmutableDeserializedPacket::new(packet)?);
        Self::new_from_immutable(vote, vote_source, deprecate_legacy_vote_ixs)
    }

    pub fn vote_pubkey(&self) -> Pubkey {
        self.vote_pubkey
    }

    pub fn slot(&self) -> Slot {
        self.slot
    }

    pub fn source(&self) -> VoteSource {
        self.vote_source
    }

    pub(crate) fn hash(&self) -> Hash {
        self.hash
    }

    pub fn timestamp(&self) -> Option<UnixTimestamp> {
        self.timestamp
    }

    pub fn is_vote_taken(&self) -> bool {
        self.vote.is_none()
    }

    pub fn take_vote(&mut self) -> Option<Arc<ImmutableDeserializedPacket>> {
        self.vote.take()
    }
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        itertools::Itertools,
        gorbagana_packet::PacketFlags,
        gorbagana_perf::packet::{BytesPacket, PacketBatch},
        gorbagana_runtime::genesis_utils::ValidatorVoteKeypairs,
        gorbagana_signer::Signer,
        gorbagana_system_transaction::transfer,
        gorbagana_vote::vote_transaction::new_tower_sync_transaction,
        gorbagana_vote_program::vote_state::TowerSync,
    };

    fn deserialize_packets(
        packet_batch: &PacketBatch,
        vote_source: VoteSource,
    ) -> impl Iterator<Item = LatestValidatorVotePacket> + '_ {
        packet_batch.iter().filter_map(move |packet| {
            LatestValidatorVotePacket::new(packet, vote_source, true).ok()
        })
    }

    #[test]
    fn test_deserialize_vote_packets() {
        let keypairs = ValidatorVoteKeypairs::new_rand();
        let blockhash = Hash::new_unique();
        let switch_proof = Hash::new_unique();
        let mut tower_sync = BytesPacket::from_data(
            None,
            new_tower_sync_transaction(
                TowerSync::from(vec![(0, 3), (1, 2), (2, 1)]),
                blockhash,
                &keypairs.node_keypair,
                &keypairs.vote_keypair,
                &keypairs.vote_keypair,
                None,
            ),
        )
        .unwrap();
        tower_sync
            .meta_mut()
            .flags
            .set(PacketFlags::SIMPLE_VOTE_TX, true);
        let mut tower_sync_switch = BytesPacket::from_data(
            None,
            new_tower_sync_transaction(
                TowerSync::from(vec![(0, 3), (1, 2), (3, 1)]),
                blockhash,
                &keypairs.node_keypair,
                &keypairs.vote_keypair,
                &keypairs.vote_keypair,
                Some(switch_proof),
            ),
        )
        .unwrap();
        tower_sync_switch
            .meta_mut()
            .flags
            .set(PacketFlags::SIMPLE_VOTE_TX, true);
        let random_transaction = BytesPacket::from_data(
            None,
            transfer(
                &keypairs.node_keypair,
                &Pubkey::new_unique(),
                1000,
                blockhash,
            ),
        )
        .unwrap();
        let packet_batch =
            PacketBatch::from(vec![tower_sync, tower_sync_switch, random_transaction]);

        let deserialized_packets =
            deserialize_packets(&packet_batch, VoteSource::Gossip).collect_vec();

        assert_eq!(2, deserialized_packets.len());
        assert_eq!(VoteSource::Gossip, deserialized_packets[0].vote_source);
        assert_eq!(VoteSource::Gossip, deserialized_packets[1].vote_source);

        assert_eq!(
            keypairs.vote_keypair.pubkey(),
            deserialized_packets[0].vote_pubkey
        );
        assert_eq!(
            keypairs.vote_keypair.pubkey(),
            deserialized_packets[1].vote_pubkey
        );

        assert!(deserialized_packets[0].vote.is_some());
        assert!(deserialized_packets[1].vote.is_some());
    }
}
