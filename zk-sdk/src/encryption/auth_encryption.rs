//! Authenticated encryption implementation.
//!
//! This module is a simple wrapper of the `Aes128GcmSiv` implementation specialized for SPL
//! token-2022 where the plaintext is always `u64`.
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use {
    crate::{
        encryption::{AE_CIPHERTEXT_LEN, AE_KEY_LEN},
        errors::AuthenticatedEncryptionError,
    },
    aes_gcm_siv::{
        aead::{Aead, KeyInit},
        Aes128GcmSiv,
    },
    base64::{prelude::BASE64_STANDARD, Engine},
    rand::{rngs::OsRng, Rng},
    std::{convert::TryInto, fmt},
    zeroize::Zeroize,
};
// Currently, `wasm_bindgen` exports types and functions included in the current crate, but all
// types and functions exported for wasm targets in all of its dependencies
// (https://github.com/rustwasm/wasm-bindgen/issues/3759). We specifically exclude some of the
// dependencies that will cause unnecessary bloat to the wasm binary.
#[cfg(not(target_arch = "wasm32"))]
use {
    sha3::Digest,
    sha3::Sha3_512,
    gorbagana_derivation_path::DerivationPath,
    gorbagana_seed_derivable::SeedDerivable,
    gorbagana_seed_phrase::generate_seed_from_seed_phrase_and_passphrase,
    gorbagana_signature::Signature,
    gorbagana_signer::{EncodableKey, Signer, SignerError},
    std::{
        error,
        io::{Read, Write},
    },
    subtle::ConstantTimeEq,
};

/// Byte length of an authenticated encryption nonce component
const NONCE_LEN: usize = 12;

/// Byte length of an authenticated encryption ciphertext component
const CIPHERTEXT_LEN: usize = 24;

struct AuthenticatedEncryption;
impl AuthenticatedEncryption {
    /// Generates an authenticated encryption key.
    ///
    /// This function is randomized. It internally samples a 128-bit key using `OsRng`.
    fn keygen() -> AeKey {
        AeKey(OsRng.gen::<[u8; AE_KEY_LEN]>())
    }

    /// On input of an authenticated encryption key and an amount, the function returns a
    /// corresponding authenticated encryption ciphertext.
    fn encrypt(key: &AeKey, balance: u64) -> AeCiphertext {
        let mut plaintext = balance.to_le_bytes();
        let nonce: Nonce = OsRng.gen::<[u8; NONCE_LEN]>();

        // The balance and the nonce have fixed length and therefore, encryption should not fail.
        let ciphertext = Aes128GcmSiv::new(&key.0.into())
            .encrypt(&nonce.into(), plaintext.as_ref())
            .expect("authenticated encryption");

        plaintext.zeroize();

        AeCiphertext {
            nonce,
            ciphertext: ciphertext.try_into().unwrap(),
        }
    }

    /// On input of an authenticated encryption key and a ciphertext, the function returns the
    /// originally encrypted amount.
    fn decrypt(key: &AeKey, ciphertext: &AeCiphertext) -> Option<u64> {
        let plaintext = Aes128GcmSiv::new(&key.0.into())
            .decrypt(&ciphertext.nonce.into(), ciphertext.ciphertext.as_ref());

        if let Ok(plaintext) = plaintext {
            let amount_bytes: [u8; 8] = plaintext.try_into().unwrap();
            Some(u64::from_le_bytes(amount_bytes))
        } else {
            None
        }
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Clone, Debug, Zeroize, Eq, PartialEq)]
pub struct AeKey([u8; AE_KEY_LEN]);
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl AeKey {
    /// Generates a random authenticated encryption key.
    ///
    /// This function is randomized. It internally samples a scalar element using `OsRng`.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = newRand))]
    pub fn new_rand() -> Self {
        AuthenticatedEncryption::keygen()
    }

    /// Encrypts an amount under the authenticated encryption key.
    pub fn encrypt(&self, amount: u64) -> AeCiphertext {
        AuthenticatedEncryption::encrypt(self, amount)
    }

    pub fn decrypt(&self, ciphertext: &AeCiphertext) -> Option<u64> {
        AuthenticatedEncryption::decrypt(self, ciphertext)
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl AeKey {
    /// Deterministically derives an authenticated encryption key from a Gorbagana signer and a public
    /// seed.
    ///
    /// This function exists for applications where a user may not wish to maintain a Gorbagana signer
    /// and an authenticated encryption key separately. Instead, a user can derive the ElGamal
    /// keypair on-the-fly whenever encrytion/decryption is needed.
    pub fn new_from_signer(
        signer: &dyn Signer,
        public_seed: &[u8],
    ) -> Result<Self, Box<dyn error::Error>> {
        let seed = Self::seed_from_signer(signer, public_seed)?;
        Self::from_seed(&seed)
    }

    /// Derive a seed from a Gorbagana signer used to generate an authenticated encryption key.
    ///
    /// The seed is derived as the hash of the signature of a public seed.
    pub fn seed_from_signer(
        signer: &dyn Signer,
        public_seed: &[u8],
    ) -> Result<Vec<u8>, SignerError> {
        let message = [b"AeKey", public_seed].concat();
        let signature = signer.try_sign_message(&message)?;

        // Some `Signer` implementations return the default signature, which is not suitable for
        // use as key material
        if bool::from(signature.as_ref().ct_eq(Signature::default().as_ref())) {
            return Err(SignerError::Custom("Rejecting default signature".into()));
        }

        Ok(Self::seed_from_signature(&signature))
    }

    /// Derive an authenticated encryption key from a signature.
    pub fn new_from_signature(signature: &Signature) -> Result<Self, Box<dyn error::Error>> {
        let seed = Self::seed_from_signature(signature);
        Self::from_seed(&seed)
    }

    /// Derive a seed from a signature used to generate an authenticated encryption key.
    pub fn seed_from_signature(signature: &Signature) -> Vec<u8> {
        let mut hasher = Sha3_512::new();
        hasher.update(signature);
        let result = hasher.finalize();

        result.to_vec()
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl EncodableKey for AeKey {
    fn read<R: Read>(reader: &mut R) -> Result<Self, Box<dyn error::Error>> {
        let bytes: [u8; AE_KEY_LEN] = serde_json::from_reader(reader)?;
        Ok(Self(bytes))
    }

    fn write<W: Write>(&self, writer: &mut W) -> Result<String, Box<dyn error::Error>> {
        let bytes = self.0;
        let json = serde_json::to_string(&bytes.to_vec())?;
        writer.write_all(&json.clone().into_bytes())?;
        Ok(json)
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl SeedDerivable for AeKey {
    fn from_seed(seed: &[u8]) -> Result<Self, Box<dyn error::Error>> {
        const MINIMUM_SEED_LEN: usize = AE_KEY_LEN;
        const MAXIMUM_SEED_LEN: usize = 65535;

        if seed.len() < MINIMUM_SEED_LEN {
            return Err(AuthenticatedEncryptionError::SeedLengthTooShort.into());
        }
        if seed.len() > MAXIMUM_SEED_LEN {
            return Err(AuthenticatedEncryptionError::SeedLengthTooLong.into());
        }

        let mut hasher = Sha3_512::new();
        hasher.update(seed);
        let result = hasher.finalize();

        Ok(Self(result[..AE_KEY_LEN].try_into()?))
    }

    fn from_seed_and_derivation_path(
        _seed: &[u8],
        _derivation_path: Option<DerivationPath>,
    ) -> Result<Self, Box<dyn error::Error>> {
        Err(AuthenticatedEncryptionError::DerivationMethodNotSupported.into())
    }

    fn from_seed_phrase_and_passphrase(
        seed_phrase: &str,
        passphrase: &str,
    ) -> Result<Self, Box<dyn error::Error>> {
        Self::from_seed(&generate_seed_from_seed_phrase_and_passphrase(
            seed_phrase,
            passphrase,
        ))
    }
}

impl From<[u8; AE_KEY_LEN]> for AeKey {
    fn from(bytes: [u8; AE_KEY_LEN]) -> Self {
        Self(bytes)
    }
}

impl From<AeKey> for [u8; AE_KEY_LEN] {
    fn from(key: AeKey) -> Self {
        key.0
    }
}

impl TryFrom<&[u8]> for AeKey {
    type Error = AuthenticatedEncryptionError;
    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() != AE_KEY_LEN {
            return Err(AuthenticatedEncryptionError::Deserialization);
        }
        bytes
            .try_into()
            .map(Self)
            .map_err(|_| AuthenticatedEncryptionError::Deserialization)
    }
}

/// For the purpose of encrypting balances for the spl token accounts, the nonce and ciphertext
/// sizes should always be fixed.
type Nonce = [u8; NONCE_LEN];
type Ciphertext = [u8; CIPHERTEXT_LEN];

/// Authenticated encryption nonce and ciphertext
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Clone, Copy, Debug, Default)]
pub struct AeCiphertext {
    nonce: Nonce,
    ciphertext: Ciphertext,
}
impl AeCiphertext {
    pub fn decrypt(&self, key: &AeKey) -> Option<u64> {
        AuthenticatedEncryption::decrypt(key, self)
    }

    pub fn to_bytes(&self) -> [u8; AE_CIPHERTEXT_LEN] {
        let mut buf = [0_u8; AE_CIPHERTEXT_LEN];
        buf[..NONCE_LEN].copy_from_slice(&self.nonce);
        buf[NONCE_LEN..].copy_from_slice(&self.ciphertext);
        buf
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<AeCiphertext> {
        if bytes.len() != AE_CIPHERTEXT_LEN {
            return None;
        }

        let nonce = bytes[..NONCE_LEN].try_into().ok()?;
        let ciphertext = bytes[NONCE_LEN..].try_into().ok()?;

        Some(AeCiphertext { nonce, ciphertext })
    }
}

impl fmt::Display for AeCiphertext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", BASE64_STANDARD.encode(self.to_bytes()))
    }
}

#[cfg(test)]
mod tests {
    use {
        super::*, gorbagana_keypair::Keypair, gorbagana_pubkey::Pubkey,
        gorbagana_signer::null_signer::NullSigner,
    };

    #[test]
    fn test_aes_encrypt_decrypt_correctness() {
        let key = AeKey::new_rand();
        let amount = 55;

        let ciphertext = key.encrypt(amount);
        let decrypted_amount = ciphertext.decrypt(&key).unwrap();

        assert_eq!(amount, decrypted_amount);
    }

    #[test]
    fn test_aes_new() {
        let keypair1 = Keypair::new();
        let keypair2 = Keypair::new();

        assert_ne!(
            AeKey::new_from_signer(&keypair1, Pubkey::default().as_ref())
                .unwrap()
                .0,
            AeKey::new_from_signer(&keypair2, Pubkey::default().as_ref())
                .unwrap()
                .0,
        );

        let null_signer = NullSigner::new(&Pubkey::default());
        assert!(AeKey::new_from_signer(&null_signer, Pubkey::default().as_ref()).is_err());
    }

    #[test]
    fn test_aes_key_from_seed() {
        let good_seed = vec![0; 32];
        assert!(AeKey::from_seed(&good_seed).is_ok());

        let too_short_seed = vec![0; 15];
        assert!(AeKey::from_seed(&too_short_seed).is_err());

        let too_long_seed = vec![0; 65536];
        assert!(AeKey::from_seed(&too_long_seed).is_err());
    }

    #[test]
    fn test_aes_key_from() {
        let key = AeKey::from_seed(&[0; 32]).unwrap();
        let key_bytes: [u8; AE_KEY_LEN] = AeKey::from_seed(&[0; 32]).unwrap().into();

        assert_eq!(key, AeKey::from(key_bytes));
    }

    #[test]
    fn test_aes_key_try_from() {
        let key = AeKey::from_seed(&[0; 32]).unwrap();
        let key_bytes: [u8; AE_KEY_LEN] = AeKey::from_seed(&[0; 32]).unwrap().into();

        assert_eq!(key, AeKey::try_from(key_bytes.as_slice()).unwrap());
    }

    #[test]
    fn test_aes_key_try_from_error() {
        let too_many_bytes = vec![0_u8; 32];
        assert!(AeKey::try_from(too_many_bytes.as_slice()).is_err());
    }
}
