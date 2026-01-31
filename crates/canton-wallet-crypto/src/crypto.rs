// Cryptographic operations for Canton Wallet SDK

use canton_wallet_core::{KeyAlgorithm, PublicKey, Signature, WalletError, WalletResult};
use ed25519_dalek::{Keypair, PublicKey as Ed25519PublicKey, SecretKey, Signer, Verifier};
use rand::rngs::OsRng;
use rand_core::RngCore;
use sha2::{Digest, Sha256};
use zeroize::Zeroize;

/// Key pair for cryptographic operations
#[derive(Debug, Clone)]
pub struct KeyPair {
    pub public_key: PublicKey,
    pub secret_key: Vec<u8>,
}

impl KeyPair {
    /// Create a new Ed25519 key pair
    pub fn new_ed25519() -> Self {
        let mut csprng = OsRng;
        let keypair = Keypair::generate(&mut csprng);
        Self {
            public_key: PublicKey::new(keypair.public.to_bytes().to_vec(), "ed25519"),
            secret_key: keypair.secret.to_bytes().to_vec(),
        }
    }

    /// Create a key pair from secret key bytes
    pub fn from_secret_key(algorithm: KeyAlgorithm, secret_key: &[u8]) -> WalletResult<Self> {
        match algorithm {
            KeyAlgorithm::Ed25519 => {
                let secret = SecretKey::from_bytes(secret_key)
                    .map_err(|e| WalletError::KeyImportFailed(e.to_string()))?;
                let public = Ed25519PublicKey::from(&secret);
                Ok(Self {
                    public_key: PublicKey::new(public.to_bytes().to_vec(), "ed25519"),
                    secret_key: secret_key.to_vec(),
                })
            }
            KeyAlgorithm::Secp256k1 | KeyAlgorithm::Secp256r1 => {
                return Err(WalletError::KeyImportFailed(format!(
                    "Algorithm {:?} not yet implemented",
                    algorithm
                )));
            }
        }
    }

    /// Get the public key
    pub fn public_key(&self) -> &PublicKey {
        &self.public_key
    }

    /// Get the secret key
    pub fn secret_key(&self) -> &[u8] {
        &self.secret_key
    }

    /// Zeroize the secret key
    pub fn zeroize(&mut self) {
        self.secret_key.zeroize();
    }
}

impl Drop for KeyPair {
    fn drop(&mut self) {
        self.zeroize();
    }
}

/// Cryptographic operations
pub struct CryptoOps;

impl CryptoOps {
    /// Generate a new key pair
    pub fn generate_keypair(algorithm: KeyAlgorithm) -> WalletResult<KeyPair> {
        match algorithm {
            KeyAlgorithm::Ed25519 => Ok(KeyPair::new_ed25519()),
            KeyAlgorithm::Secp256k1 | KeyAlgorithm::Secp256r1 => {
                Err(WalletError::KeyGenerationFailed(format!(
                    "Algorithm {:?} not yet implemented",
                    algorithm
                )))
            }
        }
    }

    /// Sign data with a key pair
    pub fn sign(keypair: &KeyPair, data: &[u8]) -> WalletResult<Signature> {
        match keypair.public_key.algorithm.as_str() {
            "ed25519" => {
                let secret = SecretKey::from_bytes(keypair.secret_key())
                    .map_err(|e| WalletError::KeyImportFailed(e.to_string()))?;
                let keypair_ed = Keypair {
                    secret,
                    public: Ed25519PublicKey::from_bytes(keypair.public_key.as_bytes())
                        .map_err(|_| WalletError::InvalidSignature)?,
                };
                let signature = keypair_ed.sign(data);
                Ok(Signature::new(signature.to_bytes().to_vec(), "ed25519"))
            }
            _ => Err(WalletError::InvalidSignature),
        }
    }

    /// Verify a signature
    pub fn verify(
        public_key: &PublicKey,
        data: &[u8],
        signature: &Signature,
    ) -> WalletResult<bool> {
        match public_key.algorithm.as_str() {
            "ed25519" => {
                let public = Ed25519PublicKey::from_bytes(public_key.as_bytes())
                    .map_err(|_| WalletError::InvalidSignature)?;
                let sig = ed25519_dalek::Signature::from_bytes(&signature.bytes)
                    .map_err(|_| WalletError::InvalidSignature)?;
                Ok(public.verify(data, &sig).is_ok())
            }
            _ => Err(WalletError::InvalidSignature),
        }
    }

    /// Hash data using SHA-256
    pub fn hash_sha256(data: &[u8]) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hasher.finalize().to_vec()
    }

    /// Hash data using BLAKE3
    pub fn hash_blake3(data: &[u8]) -> Vec<u8> {
        use blake3::Hasher;
        let mut hasher = Hasher::new();
        hasher.update(data);
        hasher.finalize().as_bytes().to_vec()
    }

    /// Derive a key using HKDF-SHA256
    pub fn derive_key_hkdf_sha256(
        ikm: &[u8],
        salt: &[u8],
        info: &[u8],
        length: usize,
    ) -> WalletResult<Vec<u8>> {
        use hkdf::Hkdf;
        use sha2::Sha256;

        let hk = Hkdf::<Sha256>::new(Some(salt), ikm)
            .map_err(|e| WalletError::KeyGenerationFailed(e.to_string()))?;
        let mut okm = vec![0u8; length];
        hk.expand(info, &mut okm)
            .map_err(|e| WalletError::KeyGenerationFailed(e.to_string()))?;
        Ok(okm)
    }

    /// Generate a random nonce
    pub fn generate_nonce(length: usize) -> Vec<u8> {
        let mut nonce = vec![0u8; length];
        let mut rng = OsRng;
        rng.fill_bytes(&mut nonce);
        nonce
    }
}

/// Generate a new key pair
pub fn generate_keypair(algorithm: KeyAlgorithm) -> WalletResult<KeyPair> {
    CryptoOps::generate_keypair(algorithm)
}

/// Sign data with a key pair
pub fn sign(keypair: &KeyPair, data: &[u8]) -> WalletResult<Signature> {
    CryptoOps::sign(keypair, data)
}

/// Verify a signature
pub fn verify(
    public_key: &PublicKey,
    data: &[u8],
    signature: &Signature,
) -> WalletResult<bool> {
    CryptoOps::verify(public_key, data, signature)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keypair_ed25519() {
        let keypair = KeyPair::new_ed25519();
        assert_eq!(keypair.public_key.algorithm, "ed25519");
        assert_eq!(keypair.secret_key.len(), 32);
        assert_eq!(keypair.public_key.as_bytes().len(), 32);
    }

    #[test]
    fn test_keypair_from_secret_key() {
        let keypair1 = KeyPair::new_ed25519();
        let secret_key = keypair1.secret_key().to_vec();
        let keypair2 = KeyPair::from_secret_key(KeyAlgorithm::Ed25519, &secret_key).unwrap();
        assert_eq!(keypair1.public_key, keypair2.public_key);
    }

    #[test]
    fn test_sign_verify() {
        let keypair = KeyPair::new_ed25519();
        let data = b"test data";
        let signature = sign(&keypair, data).unwrap();
        let verified = verify(&keypair.public_key, data, &signature).unwrap();
        assert!(verified);
    }

    #[test]
    fn test_sign_verify_invalid() {
        let keypair = KeyPair::new_ed25519();
        let data = b"test data";
        let signature = sign(&keypair, data).unwrap();
        let wrong_data = b"wrong data";
        let verified = verify(&keypair.public_key, wrong_data, &signature).unwrap();
        assert!(!verified);
    }

    #[test]
    fn test_hash_sha256() {
        let data = b"test data";
        let hash = CryptoOps::hash_sha256(data);
        assert_eq!(hash.len(), 32);
    }

    #[test]
    fn test_hash_blake3() {
        let data = b"test data";
        let hash = CryptoOps::hash_blake3(data);
        assert_eq!(hash.len(), 32);
    }

    #[test]
    fn test_derive_key_hkdf_sha256() {
        let ikm = b"input key material";
        let salt = b"salt";
        let info = b"info";
        let key = CryptoOps::derive_key_hkdf_sha256(ikm, salt, info, 32).unwrap();
        assert_eq!(key.len(), 32);
    }

    #[test]
    fn test_generate_nonce() {
        let nonce1 = CryptoOps::generate_nonce(16);
        let nonce2 = CryptoOps::generate_nonce(16);
        assert_eq!(nonce1.len(), 16);
        assert_eq!(nonce2.len(), 16);
        assert_ne!(nonce1, nonce2);
    }

    #[test]
    fn test_keypair_zeroize() {
        let mut keypair = KeyPair::new_ed25519();
        let secret_key = keypair.secret_key().to_vec();
        keypair.zeroize();
        assert_ne!(keypair.secret_key, secret_key);
    }
}
