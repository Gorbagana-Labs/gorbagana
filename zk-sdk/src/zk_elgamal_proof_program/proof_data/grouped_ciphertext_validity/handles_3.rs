//! The grouped-ciphertext with 3 decryption handles validity proof instruction.
//!
//! A grouped-ciphertext validity proof certifies that a grouped ElGamal ciphertext is
//! well-defined, i.e. the ciphertext can be decrypted by private keys associated with its
//! decryption handles. To generate the proof, a prover must provide the Pedersen opening
//! associated with the grouped ciphertext's commitment.

#[cfg(target_arch = "wasm32")]
use {
    crate::encryption::grouped_elgamal::GroupedElGamalCiphertext3Handles, wasm_bindgen::prelude::*,
};
use {
    crate::{
        encryption::pod::{
            elgamal::PodElGamalPubkey, grouped_elgamal::PodGroupedElGamalCiphertext3Handles,
        },
        sigma_proofs::pod::PodGroupedCiphertext3HandlesValidityProof,
        zk_elgamal_proof_program::proof_data::{pod::impl_wasm_to_bytes, ProofType, ZkProofData},
    },
    bytemuck_derive::{Pod, Zeroable},
};
#[cfg(not(target_os = "gorbagana"))]
use {
    crate::{
        encryption::{
            elgamal::ElGamalPubkey, grouped_elgamal::GroupedElGamalCiphertext,
            pedersen::PedersenOpening,
        },
        sigma_proofs::grouped_ciphertext_validity::GroupedCiphertext3HandlesValidityProof,
        zk_elgamal_proof_program::{
            errors::{ProofGenerationError, ProofVerificationError},
            proof_data::errors::ProofDataError,
        },
    },
    bytemuck::bytes_of,
    merlin::Transcript,
};

/// The instruction data that is needed for the
/// `ProofInstruction::VerifyGroupedCiphertext3HandlesValidity` instruction.
///
/// It includes the cryptographic proof as well as the context data information needed to verify
/// the proof.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct GroupedCiphertext3HandlesValidityProofData {
    pub context: GroupedCiphertext3HandlesValidityProofContext,

    pub proof: PodGroupedCiphertext3HandlesValidityProof,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct GroupedCiphertext3HandlesValidityProofContext {
    pub first_pubkey: PodElGamalPubkey, // 32 bytes

    pub second_pubkey: PodElGamalPubkey, // 32 bytes

    pub third_pubkey: PodElGamalPubkey, // 32 bytes

    pub grouped_ciphertext: PodGroupedElGamalCiphertext3Handles, // 128 bytes
}

#[cfg(not(target_os = "gorbagana"))]
#[cfg(not(target_arch = "wasm32"))]
impl GroupedCiphertext3HandlesValidityProofData {
    pub fn new(
        first_pubkey: &ElGamalPubkey,
        second_pubkey: &ElGamalPubkey,
        third_pubkey: &ElGamalPubkey,
        grouped_ciphertext: &GroupedElGamalCiphertext<3>,
        amount: u64,
        opening: &PedersenOpening,
    ) -> Result<Self, ProofGenerationError> {
        let pod_first_pubkey = PodElGamalPubkey(first_pubkey.into());
        let pod_second_pubkey = PodElGamalPubkey(second_pubkey.into());
        let pod_third_pubkey = PodElGamalPubkey(third_pubkey.into());
        let pod_grouped_ciphertext = (*grouped_ciphertext).into();

        let context = GroupedCiphertext3HandlesValidityProofContext {
            first_pubkey: pod_first_pubkey,
            second_pubkey: pod_second_pubkey,
            third_pubkey: pod_third_pubkey,
            grouped_ciphertext: pod_grouped_ciphertext,
        };

        let mut transcript = context.new_transcript();

        let proof = GroupedCiphertext3HandlesValidityProof::new(
            first_pubkey,
            second_pubkey,
            third_pubkey,
            amount,
            opening,
            &mut transcript,
        )
        .into();

        Ok(Self { context, proof })
    }
}

// Define a separate constructor for `wasm32` target since `wasm_bindgen` does
// not yet support parameters with generic constants (i.e.
// `GroupedElGamalCiphertext<3>`).
#[cfg(target_arch = "wasm32")]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl GroupedCiphertext3HandlesValidityProofData {
    pub fn new(
        first_pubkey: &ElGamalPubkey,
        second_pubkey: &ElGamalPubkey,
        third_pubkey: &ElGamalPubkey,
        grouped_ciphertext: &GroupedElGamalCiphertext3Handles,
        amount: u64,
        opening: &PedersenOpening,
    ) -> Result<Self, ProofGenerationError> {
        let pod_first_pubkey = PodElGamalPubkey(first_pubkey.into());
        let pod_second_pubkey = PodElGamalPubkey(second_pubkey.into());
        let pod_third_pubkey = PodElGamalPubkey(third_pubkey.into());
        let pod_grouped_ciphertext = grouped_ciphertext.0.into();

        let context = GroupedCiphertext3HandlesValidityProofContext {
            first_pubkey: pod_first_pubkey,
            second_pubkey: pod_second_pubkey,
            third_pubkey: pod_third_pubkey,
            grouped_ciphertext: pod_grouped_ciphertext,
        };

        let mut transcript = context.new_transcript();

        let proof = GroupedCiphertext3HandlesValidityProof::new(
            first_pubkey,
            second_pubkey,
            third_pubkey,
            amount,
            opening,
            &mut transcript,
        )
        .into();

        Ok(Self { context, proof })
    }
}

impl_wasm_to_bytes!(TYPE = GroupedCiphertext3HandlesValidityProofData);

impl ZkProofData<GroupedCiphertext3HandlesValidityProofContext>
    for GroupedCiphertext3HandlesValidityProofData
{
    const PROOF_TYPE: ProofType = ProofType::GroupedCiphertext3HandlesValidity;

    fn context_data(&self) -> &GroupedCiphertext3HandlesValidityProofContext {
        &self.context
    }

    #[cfg(not(target_os = "gorbagana"))]
    fn verify_proof(&self) -> Result<(), ProofVerificationError> {
        let mut transcript = self.context.new_transcript();

        let first_pubkey = self.context.first_pubkey.try_into()?;
        let second_pubkey = self.context.second_pubkey.try_into()?;
        let third_pubkey = self.context.third_pubkey.try_into()?;
        let grouped_ciphertext: GroupedElGamalCiphertext<3> =
            self.context.grouped_ciphertext.try_into()?;

        let first_handle = grouped_ciphertext.handles.first().unwrap();
        let second_handle = grouped_ciphertext.handles.get(1).unwrap();
        let third_handle = grouped_ciphertext.handles.get(2).unwrap();

        let proof: GroupedCiphertext3HandlesValidityProof = self.proof.try_into()?;

        proof
            .verify(
                &grouped_ciphertext.commitment,
                &first_pubkey,
                &second_pubkey,
                &third_pubkey,
                first_handle,
                second_handle,
                third_handle,
                &mut transcript,
            )
            .map_err(|e| e.into())
    }
}

#[cfg(not(target_os = "gorbagana"))]
impl GroupedCiphertext3HandlesValidityProofContext {
    fn new_transcript(&self) -> Transcript {
        let mut transcript = Transcript::new(b"grouped-ciphertext-validity-3-handles-instruction");

        transcript.append_message(b"first-pubkey", bytes_of(&self.first_pubkey));
        transcript.append_message(b"second-pubkey", bytes_of(&self.second_pubkey));
        transcript.append_message(b"third-pubkey", bytes_of(&self.third_pubkey));
        transcript.append_message(b"grouped-ciphertext", bytes_of(&self.grouped_ciphertext));

        transcript
    }
}

impl_wasm_to_bytes!(TYPE = GroupedCiphertext3HandlesValidityProofContext);

#[cfg(test)]
mod test {
    use {
        super::*,
        crate::encryption::{elgamal::ElGamalKeypair, grouped_elgamal::GroupedElGamal},
    };

    #[test]
    fn test_ciphertext_validity_proof_instruction_correctness() {
        let first_keypair = ElGamalKeypair::new_rand();
        let first_pubkey = first_keypair.pubkey();

        let second_keypair = ElGamalKeypair::new_rand();
        let second_pubkey = second_keypair.pubkey();

        let third_keypair = ElGamalKeypair::new_rand();
        let third_pubkey = third_keypair.pubkey();

        let amount: u64 = 55;
        let opening = PedersenOpening::new_rand();
        let grouped_ciphertext = GroupedElGamal::encrypt_with(
            [first_pubkey, second_pubkey, third_pubkey],
            amount,
            &opening,
        );

        let proof_data = GroupedCiphertext3HandlesValidityProofData::new(
            first_pubkey,
            second_pubkey,
            third_pubkey,
            &grouped_ciphertext,
            amount,
            &opening,
        )
        .unwrap();

        assert!(proof_data.verify_proof().is_ok());
    }
}
