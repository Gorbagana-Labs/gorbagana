//! The Bulletproofs range-proof implementation over Curve25519 Ristretto points.
//!
//! The implementation is based on the dalek-cryptography bulletproofs
//! [implementation](https://github.com/dalek-cryptography/bulletproofs). Compared to the original
//! implementation by dalek-cryptography:
//! - This implementation focuses on the range proof implementation, while the dalek-cryptography
//!   crate additionally implements the general bulletproofs implementation for languages that can be
//!   represented by arithmetic circuits as well as MPC.
//! - This implementation implements a non-interactive range proof aggregation that is specified in
//!   the original Bulletproofs [paper](https://eprint.iacr.org/2017/1066) (Section 4.3).
//!

#![allow(dead_code)]

use crate::{RISTRETTO_POINT_LEN, SCALAR_LEN};
#[cfg(not(target_os = "gorbagana"))]
use {
    crate::{
        encryption::pedersen::{Pedersen, PedersenCommitment, PedersenOpening, G, H},
        range_proof::{
            errors::{RangeProofGenerationError, RangeProofVerificationError},
            generators::RangeProofGens,
            inner_product::InnerProductProof,
        },
        transcript::TranscriptProtocol,
    },
    core::iter,
    curve25519_dalek::traits::MultiscalarMul,
    curve25519_dalek::{
        ristretto::{CompressedRistretto, RistrettoPoint},
        scalar::Scalar,
        traits::{IsIdentity, VartimeMultiscalarMul},
    },
    merlin::Transcript,
    rand::rngs::OsRng,
    subtle::{Choice, ConditionallySelectable},
};

pub mod errors;
pub mod pod;

#[cfg(not(target_os = "gorbagana"))]
pub mod generators;
#[cfg(not(target_os = "gorbagana"))]
pub mod inner_product;
#[cfg(not(target_os = "gorbagana"))]
pub mod util;

/// Byte length of a range proof excluding the inner-product proof component
pub const RANGE_PROOF_MODULO_INNER_PRODUCT_PROOF_LEN: usize =
    5 * RISTRETTO_POINT_LEN + 2 * SCALAR_LEN;

/// Byte length of an inner-product proof for a vector of length 64
pub const INNER_PRODUCT_PROOF_U64_LEN: usize = 448;

/// Byte length of a range proof for an unsigned 64-bit number
pub const RANGE_PROOF_U64_LEN: usize =
    INNER_PRODUCT_PROOF_U64_LEN + RANGE_PROOF_MODULO_INNER_PRODUCT_PROOF_LEN; // 672 bytes

/// Byte length of an inner-product proof for a vector of length 128
pub const INNER_PRODUCT_PROOF_U128_LEN: usize = 512;

/// Byte length of a range proof for an unsigned 128-bit number
pub const RANGE_PROOF_U128_LEN: usize =
    INNER_PRODUCT_PROOF_U128_LEN + RANGE_PROOF_MODULO_INNER_PRODUCT_PROOF_LEN; // 736 bytes

/// Byte length of an inner-product proof for a vector of length 256
pub const INNER_PRODUCT_PROOF_U256_LEN: usize = 576;

/// Byte length of a range proof for an unsigned 256-bit number
pub const RANGE_PROOF_U256_LEN: usize =
    INNER_PRODUCT_PROOF_U256_LEN + RANGE_PROOF_MODULO_INNER_PRODUCT_PROOF_LEN; // 800 bytes

#[allow(non_snake_case)]
#[cfg(not(target_os = "gorbagana"))]
#[derive(Clone)]
pub struct RangeProof {
    pub(crate) A: CompressedRistretto,       // 32 bytes
    pub(crate) S: CompressedRistretto,       // 32 bytes
    pub(crate) T_1: CompressedRistretto,     // 32 bytes
    pub(crate) T_2: CompressedRistretto,     // 32 bytes
    pub(crate) t_x: Scalar,                  // 32 bytes
    pub(crate) t_x_blinding: Scalar,         // 32 bytes
    pub(crate) e_blinding: Scalar,           // 32 bytes
    pub(crate) ipp_proof: InnerProductProof, // 448 bytes for withdraw; 512 for transfer
}

#[allow(non_snake_case)]
#[cfg(not(target_os = "gorbagana"))]
impl RangeProof {
    /// Create an aggregated range proof.
    ///
    /// The proof is created with respect to a vector of Pedersen commitments C_1, ..., C_m. The
    /// method itself does not take in these commitments, but the values associated with the commitments:
    /// - vector of committed amounts v_1, ..., v_m represented as u64
    /// - bit-lengths of the committed amounts
    /// - Pedersen openings for each commitments
    ///
    /// The sum of the bit-lengths of the commitments amounts must be a power-of-two
    #[allow(clippy::many_single_char_names)]
    #[cfg(not(target_os = "gorbagana"))]
    pub fn new(
        amounts: Vec<u64>,
        bit_lengths: Vec<usize>,
        openings: Vec<&PedersenOpening>,
        transcript: &mut Transcript,
    ) -> Result<Self, RangeProofGenerationError> {
        // amounts, bit-lengths, openings must be same length vectors
        let m = amounts.len();
        if bit_lengths.len() != m || openings.len() != m {
            return Err(RangeProofGenerationError::VectorLengthMismatch);
        }

        // each bit length must be greater than 0 for the proof to make sense
        if bit_lengths
            .iter()
            .any(|bit_length| *bit_length == 0 || *bit_length > u64::BITS as usize)
        {
            return Err(RangeProofGenerationError::InvalidBitSize);
        }

        // total vector dimension to compute the ultimate inner product proof for
        let nm: usize = bit_lengths.iter().sum();
        if !nm.is_power_of_two() {
            return Err(RangeProofGenerationError::VectorLengthMismatch);
        }

        let bp_gens = RangeProofGens::new(nm)
            .map_err(|_| RangeProofGenerationError::MaximumGeneratorLengthExceeded)?;

        transcript.range_proof_domain_separator(nm as u64);

        // bit-decompose values and generate their Pedersen vector commitment
        let a_blinding = Scalar::random(&mut OsRng);
        let mut A = a_blinding * &(*H);

        let mut gens_iter = bp_gens.G(nm).zip(bp_gens.H(nm));
        for (amount_i, n_i) in amounts.iter().zip(bit_lengths.iter()) {
            for j in 0..(*n_i) {
                let (G_ij, H_ij) = gens_iter.next().unwrap();

                // `j` is guaranteed to be at most `u64::BITS` (a 6-bit number) and therefore,
                // casting is lossless and right shift can be safely unwrapped
                let v_ij = Choice::from((amount_i.checked_shr(j as u32).unwrap() & 1) as u8);
                let mut point = -H_ij;
                point.conditional_assign(G_ij, v_ij);
                A += point;
            }
        }
        let A = A.compress();

        // generate blinding factors and generate their Pedersen vector commitment
        let s_L: Vec<Scalar> = (0..nm).map(|_| Scalar::random(&mut OsRng)).collect();
        let s_R: Vec<Scalar> = (0..nm).map(|_| Scalar::random(&mut OsRng)).collect();

        // generate blinding factor for Pedersen commitment; `s_blinding` should not to be confused
        // with blinding factors for the actual inner product vector
        let s_blinding = Scalar::random(&mut OsRng);

        let S = RistrettoPoint::multiscalar_mul(
            iter::once(&s_blinding).chain(s_L.iter()).chain(s_R.iter()),
            iter::once(&(*H)).chain(bp_gens.G(nm)).chain(bp_gens.H(nm)),
        )
        .compress();

        // add the Pedersen vector commitments to the transcript (send the commitments to the verifier)
        transcript.append_point(b"A", &A);
        transcript.append_point(b"S", &S);

        // derive challenge scalars from the transcript (receive challenge from the verifier): `y`
        // and `z` used for merge multiple inner product relations into one single inner product
        let y = transcript.challenge_scalar(b"y");
        let z = transcript.challenge_scalar(b"z");

        // define blinded vectors:
        // - l(x) = (a_L - z*1) + s_L*x
        // - r(x) = (y^n * (a_R + z*1) + [z^2*2^n | z^3*2^n | ... | z^m*2^n]) + y^n * s_R*x
        let mut l_poly = util::VecPoly1::zero(nm);
        let mut r_poly = util::VecPoly1::zero(nm);

        let mut i = 0;
        let mut exp_z = z * z;
        let mut exp_y = Scalar::ONE;

        for (amount_i, n_i) in amounts.iter().zip(bit_lengths.iter()) {
            let mut exp_2 = Scalar::ONE;

            for j in 0..(*n_i) {
                // `j` is guaranteed to be at most `u64::BITS` (a 6-bit number) and therefore,
                // casting is lossless and right shift can be safely unwrapped
                let a_L_j = Scalar::from(amount_i.checked_shr(j as u32).unwrap() & 1);
                let a_R_j = a_L_j - Scalar::ONE;

                l_poly.0[i] = a_L_j - z;
                l_poly.1[i] = s_L[i];
                r_poly.0[i] = exp_y * (a_R_j + z) + exp_z * exp_2;
                r_poly.1[i] = exp_y * s_R[i];

                exp_y *= y;
                exp_2 = exp_2 + exp_2;

                // `i` is capped by the sum of vectors in `bit_lengths`
                i = i.checked_add(1).unwrap();
            }
            exp_z *= z;
        }

        // define t(x) = <l(x), r(x)> = t_0 + t_1*x + t_2*x
        let t_poly = l_poly
            .inner_product(&r_poly)
            .ok_or(RangeProofGenerationError::InnerProductLengthMismatch)?;

        // generate Pedersen commitment for the coefficients t_1 and t_2
        let (T_1, t_1_blinding) = Pedersen::new(t_poly.1);
        let (T_2, t_2_blinding) = Pedersen::new(t_poly.2);

        let T_1 = T_1.get_point().compress();
        let T_2 = T_2.get_point().compress();

        transcript.append_point(b"T_1", &T_1);
        transcript.append_point(b"T_2", &T_2);

        // evaluate t(x) on challenge x and homomorphically compute the openings for
        // z^2 * V_1 + z^3 * V_2 + ... + z^{m+1} * V_m + delta(y, z)*G + x*T_1 + x^2*T_2
        let x = transcript.challenge_scalar(b"x");

        let mut agg_opening = Scalar::ZERO;
        let mut exp_z = z;
        for opening in openings {
            exp_z *= z;
            agg_opening += exp_z * opening.get_scalar();
        }

        let t_blinding_poly = util::Poly2(
            agg_opening,
            *t_1_blinding.get_scalar(),
            *t_2_blinding.get_scalar(),
        );

        let t_x = t_poly.eval(x);
        let t_x_blinding = t_blinding_poly.eval(x);

        transcript.append_scalar(b"t_x", &t_x);
        transcript.append_scalar(b"t_x_blinding", &t_x_blinding);

        // homomorphically compuate the openings for A + x*S
        let e_blinding = a_blinding + s_blinding * x;
        let l_vec = l_poly.eval(x);
        let r_vec = r_poly.eval(x);

        transcript.append_scalar(b"e_blinding", &e_blinding);

        // compute the inner product argument on the commitment:
        // P = <l(x), G> + <r(x), H'> + <l(x), r(x)>*Q
        let w = transcript.challenge_scalar(b"w");
        let Q = w * &G;

        let G_factors: Vec<Scalar> = iter::repeat_n(Scalar::ONE, nm).collect();
        let H_factors: Vec<Scalar> = util::exp_iter(y.invert()).take(nm).collect();

        // generate challenge `c` for consistency with the verifier's transcript
        transcript.challenge_scalar(b"c");

        let ipp_proof = InnerProductProof::new(
            &Q,
            &G_factors,
            &H_factors,
            bp_gens.G(nm).cloned().collect(),
            bp_gens.H(nm).cloned().collect(),
            l_vec,
            r_vec,
            transcript,
        )?;

        Ok(RangeProof {
            A,
            S,
            T_1,
            T_2,
            t_x,
            t_x_blinding,
            e_blinding,
            ipp_proof,
        })
    }

    #[allow(clippy::many_single_char_names)]
    pub fn verify(
        &self,
        comms: Vec<&PedersenCommitment>,
        bit_lengths: Vec<usize>,
        transcript: &mut Transcript,
    ) -> Result<(), RangeProofVerificationError> {
        // commitments and bit-lengths must be same length vectors
        if comms.len() != bit_lengths.len() {
            return Err(RangeProofVerificationError::VectorLengthMismatch);
        }

        let m = bit_lengths.len();
        let nm: usize = bit_lengths.iter().sum();
        let bp_gens = RangeProofGens::new(nm)
            .map_err(|_| RangeProofVerificationError::MaximumGeneratorLengthExceeded)?;

        if !nm.is_power_of_two() {
            return Err(RangeProofVerificationError::InvalidBitSize);
        }

        transcript.range_proof_domain_separator(nm as u64);

        // append proof data to transcript and derive appropriate challenge scalars
        transcript.validate_and_append_point(b"A", &self.A)?;
        transcript.validate_and_append_point(b"S", &self.S)?;

        let y = transcript.challenge_scalar(b"y");
        let z = transcript.challenge_scalar(b"z");

        let zz = z * z;
        let minus_z = -z;

        transcript.validate_and_append_point(b"T_1", &self.T_1)?;
        transcript.validate_and_append_point(b"T_2", &self.T_2)?;

        let x = transcript.challenge_scalar(b"x");

        transcript.append_scalar(b"t_x", &self.t_x);
        transcript.append_scalar(b"t_x_blinding", &self.t_x_blinding);
        transcript.append_scalar(b"e_blinding", &self.e_blinding);

        let w = transcript.challenge_scalar(b"w");

        // this variable exists for backwards compatibility
        let _c = transcript.challenge_scalar(b"c");

        // verify inner product proof
        let (x_sq, x_inv_sq, s) = self.ipp_proof.verification_scalars(nm, transcript)?;
        let s_inv = s.iter().rev();

        let a = self.ipp_proof.a;
        let b = self.ipp_proof.b;

        transcript.append_scalar(b"ipp_a", &a);
        transcript.append_scalar(b"ipp_b", &b);

        let d = transcript.challenge_scalar(b"d"); // challenge value for batching multiscalar mul

        // construct concat_z_and_2, an iterator of the values of
        // z^0 * \vec(2)^n || z^1 * \vec(2)^n || ... || z^(m-1) * \vec(2)^n
        let concat_z_and_2: Vec<Scalar> = util::exp_iter(z)
            .zip(bit_lengths.iter())
            .flat_map(|(exp_z, n_i)| {
                util::exp_iter(Scalar::from(2u64))
                    .take(*n_i)
                    .map(move |exp_2| exp_2 * exp_z)
            })
            .collect();

        let gs = s.iter().map(|s_i| minus_z - a * s_i);
        let hs = s_inv
            .zip(util::exp_iter(y.invert()))
            .zip(concat_z_and_2.iter())
            .map(|((s_i_inv, exp_y_inv), z_and_2)| z + exp_y_inv * (zz * z_and_2 - b * s_i_inv));

        let basepoint_scalar =
            w * (self.t_x - a * b) + d * (delta(&bit_lengths, &y, &z) - self.t_x);
        let value_commitment_scalars = util::exp_iter(z).take(m).map(|z_exp| d * zz * z_exp);

        let mega_check = RistrettoPoint::optional_multiscalar_mul(
            iter::once(Scalar::ONE)
                .chain(iter::once(x))
                .chain(iter::once(d * x))
                .chain(iter::once(d * x * x))
                .chain(iter::once(-self.e_blinding - d * self.t_x_blinding))
                .chain(iter::once(basepoint_scalar))
                .chain(x_sq.iter().cloned())
                .chain(x_inv_sq.iter().cloned())
                .chain(gs)
                .chain(hs)
                .chain(value_commitment_scalars),
            iter::once(self.A.decompress())
                .chain(iter::once(self.S.decompress()))
                .chain(iter::once(self.T_1.decompress()))
                .chain(iter::once(self.T_2.decompress()))
                .chain(iter::once(Some(*H)))
                .chain(iter::once(Some(G)))
                .chain(self.ipp_proof.L_vec.iter().map(|L| L.decompress()))
                .chain(self.ipp_proof.R_vec.iter().map(|R| R.decompress()))
                .chain(bp_gens.G(nm).map(|&x| Some(x)))
                .chain(bp_gens.H(nm).map(|&x| Some(x)))
                .chain(comms.iter().map(|V| Some(*V.get_point()))),
        )
        .ok_or(RangeProofVerificationError::MultiscalarMul)?;

        if mega_check.is_identity() {
            Ok(())
        } else {
            Err(RangeProofVerificationError::AlgebraicRelation)
        }
    }

    // Following the dalek rangeproof library signature for now. The exact method signature can be
    // changed.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(7 * 32 + self.ipp_proof.serialized_size());
        buf.extend_from_slice(self.A.as_bytes());
        buf.extend_from_slice(self.S.as_bytes());
        buf.extend_from_slice(self.T_1.as_bytes());
        buf.extend_from_slice(self.T_2.as_bytes());
        buf.extend_from_slice(self.t_x.as_bytes());
        buf.extend_from_slice(self.t_x_blinding.as_bytes());
        buf.extend_from_slice(self.e_blinding.as_bytes());
        buf.extend_from_slice(&self.ipp_proof.to_bytes());
        buf
    }

    // Following the dalek rangeproof library signature for now. The exact method signature can be
    // changed.
    pub fn from_bytes(slice: &[u8]) -> Result<RangeProof, RangeProofVerificationError> {
        if slice.len() % 32 != 0 {
            return Err(RangeProofVerificationError::Deserialization);
        }
        if slice.len() < 7 * 32 {
            return Err(RangeProofVerificationError::Deserialization);
        }

        let A = CompressedRistretto(util::read32(&slice[0..]));
        let S = CompressedRistretto(util::read32(&slice[32..]));
        let T_1 = CompressedRistretto(util::read32(&slice[2 * 32..]));
        let T_2 = CompressedRistretto(util::read32(&slice[3 * 32..]));

        let t_x = Scalar::from_canonical_bytes(util::read32(&slice[4 * 32..]))
            .into_option()
            .ok_or(RangeProofVerificationError::Deserialization)?;
        let t_x_blinding = Scalar::from_canonical_bytes(util::read32(&slice[5 * 32..]))
            .into_option()
            .ok_or(RangeProofVerificationError::Deserialization)?;
        let e_blinding = Scalar::from_canonical_bytes(util::read32(&slice[6 * 32..]))
            .into_option()
            .ok_or(RangeProofVerificationError::Deserialization)?;

        let ipp_proof = InnerProductProof::from_bytes(&slice[7 * 32..])?;

        Ok(RangeProof {
            A,
            S,
            T_1,
            T_2,
            t_x,
            t_x_blinding,
            e_blinding,
            ipp_proof,
        })
    }
}

/// Compute
/// \\[
/// \delta(y,z) = (z - z^{2}) \langle \mathbf{1}, {\mathbf{y}}^{n \cdot m} \rangle - \sum_{j=0}^{m-1} z^{j+3} \cdot \langle \mathbf{1}, {\mathbf{2}}^{n \cdot m} \rangle
/// \\]
#[cfg(not(target_os = "gorbagana"))]
fn delta(bit_lengths: &[usize], y: &Scalar, z: &Scalar) -> Scalar {
    let nm: usize = bit_lengths.iter().sum();
    let sum_y = util::sum_of_powers(y, nm);

    let mut agg_delta = (z - z * z) * sum_y;
    let mut exp_z = z * z * z;
    for n_i in bit_lengths.iter() {
        let sum_2 = util::sum_of_powers(&Scalar::from(2u64), *n_i);
        agg_delta -= exp_z * sum_2;
        exp_z *= z;
    }
    agg_delta
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        crate::{
            encryption::pod::pedersen::PodPedersenCommitment, range_proof::pod::PodRangeProofU128,
        },
        std::str::FromStr,
    };

    #[test]
    fn test_single_rangeproof() {
        let (comm, open) = Pedersen::new(55_u64);

        let mut transcript_create = Transcript::new(b"Test");
        let mut transcript_verify = Transcript::new(b"Test");

        let proof =
            RangeProof::new(vec![55], vec![32], vec![&open], &mut transcript_create).unwrap();

        proof
            .verify(vec![&comm], vec![32], &mut transcript_verify)
            .unwrap();
    }

    #[test]
    fn test_aggregated_rangeproof() {
        let (comm_1, open_1) = Pedersen::new(55_u64);
        let (comm_2, open_2) = Pedersen::new(77_u64);
        let (comm_3, open_3) = Pedersen::new(99_u64);

        let mut transcript_create = Transcript::new(b"Test");
        let mut transcript_verify = Transcript::new(b"Test");

        let proof = RangeProof::new(
            vec![55, 77, 99],
            vec![64, 32, 32],
            vec![&open_1, &open_2, &open_3],
            &mut transcript_create,
        )
        .unwrap();

        proof
            .verify(
                vec![&comm_1, &comm_2, &comm_3],
                vec![64, 32, 32],
                &mut transcript_verify,
            )
            .unwrap();
    }

    #[test]
    fn test_range_proof_string() {
        let commitment_1_str = "dDaa/MTEDlyI0Nxx+iu1tOteZsTWmPXAfn9QI0W9mSc=";
        let pod_commitment_1 = PodPedersenCommitment::from_str(commitment_1_str).unwrap();
        let commitment_1: PedersenCommitment = pod_commitment_1.try_into().unwrap();

        let commitment_2_str = "tnRILjKpogi2sXxLgZzMqlqPMLnCJmrSjZ5SPQYhtgg=";
        let pod_commitment_2 = PodPedersenCommitment::from_str(commitment_2_str).unwrap();
        let commitment_2: PedersenCommitment = pod_commitment_2.try_into().unwrap();

        let commitment_3_str = "ZAC5ZLXotsMOVExtrr56D/EZNeyo9iWepNbeH22EuRo=";
        let pod_commitment_3 = PodPedersenCommitment::from_str(commitment_3_str).unwrap();
        let commitment_3: PedersenCommitment = pod_commitment_3.try_into().unwrap();

        let proof_str = "AvvBQL63pXMXsmuvuNbs/CqXdzeyrMpEIO2O/cI6/SyqU4N+7HUU3LmXai9st+DxqTnuKsm0SgnADfpLpQCEbDDupMb09NY8oHT8Bx8WQhv9eyoBlrPRd7DVhOUsio02gBshe3p2Wj7+yDCpFaZ7/PMypFBX6+E+EqCiPI6yUk4ztslWY0Ksac41eJgcPzXyIx2kvmSTsVBKLb7U01PWBC+AUyUmK3/IdvmJ4DnlS3xFrdg/mxSsYJFd3OZA3cwDb0jePQf/P43/2VVqPRixMVO7+VGoMKPoRTEEVbClsAlW6stGTFPcrimu3c+geASgvwElkIKNGtYcjoj3SS+/VeqIG9Ei1j+TJtPhOE9SG4KNw9xBGwecpliDbQhKjO950EVcnOts+a525/frZV1jHJmOOrZtKRV4pvk37dtQkx4sv+pxRmfVrjwOcKQeg+BzcuF0vaQbqa4SUbzbO9z3RwIMlYIBaz0bqZgJmtPOFuFmNyCJaeB29vlcEAfYbn5gdlgtWP50tKmhoskndulziKZjz4qHSA9rbG2ZtoMHoCsAobHKu2H9OxcaK4Scj1QGwst+zXBEY8uePNbxvU5DMJLVFORtLUXkVdPCmCSsm1Bz4TRbnls8LOVW6wqTgShQMhjNM3RtwdHXENPn5uDnhyvfduAcL+DtI8AIJyRneROefk7i7gjal8dLdMM/QnXT7ctpMQU6uNlpsNzq65xlOQKXO71vQ3c2mE/DmxVJi6BTS5WCzavvhiqdhQyRL61ESCALQpaP0/d0DLwLikVH3ypuDLEnVXe9Pmkxdd0xCzO6QcfyK50CPnV/dVgHeLg8EVag2O83+/7Ys5oLxrDad9TJTDcrT2xsRqECFnSA+z9uZtDPujhQL0ogS5RH4agnQN4mVGTwOLV8OKpn+AvWq6+j1/9EXFkLPBTU5wT0FQuT2VZ8xp5GeqdI13Zey1uPrxc6CZZ407y9OINED4IdBQ==";
        let pod_proof = PodRangeProofU128::from_str(proof_str).unwrap();
        let proof: RangeProof = pod_proof.try_into().unwrap();

        let mut transcript_verify = Transcript::new(b"Test");

        proof
            .verify(
                vec![&commitment_1, &commitment_2, &commitment_3],
                vec![64, 32, 32],
                &mut transcript_verify,
            )
            .unwrap()
    }
}
