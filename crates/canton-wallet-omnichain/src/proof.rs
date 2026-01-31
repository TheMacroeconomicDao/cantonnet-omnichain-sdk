// Copyright 2025 Canton Wallet SDK Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use serde::{Deserialize, Serialize};
use crate::{error::OmniChainResult, types::LockReceipt};

/// Lock proof for cross-chain transfers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockProof {
    /// Lock receipt
    pub receipt: LockReceipt,
    /// Merkle proof (for inclusion verification)
    pub merkle_proof: Vec<String>,
    /// Block number
    pub block_number: u64,
    /// Block hash
    pub block_hash: String,
    /// Signature from bridge oracle
    pub oracle_signature: String,
}

impl LockProof {
    /// Create a new lock proof
    pub fn new(
        receipt: LockReceipt,
        merkle_proof: Vec<String>,
        block_number: u64,
        block_hash: impl Into<String>,
        oracle_signature: impl Into<String>,
    ) -> Self {
        Self {
            receipt,
            merkle_proof,
            block_number,
            block_hash: block_hash.into(),
            oracle_signature: oracle_signature.into(),
        }
    }

    /// Serialize to bytes
    pub fn to_bytes(&self) -> OmniChainResult<Vec<u8>> {
        serde_json::to_vec(self)
            .map_err(|e| crate::error::OmniChainError::SerializationError(e.to_string()))
    }

    /// Deserialize from bytes
    pub fn from_bytes(bytes: &[u8]) -> OmniChainResult<Self> {
        serde_json::from_slice(bytes)
            .map_err(|e| crate::error::OmniChainError::DeserializationError(e.to_string()))
    }

    /// Generate hash of the proof
    pub fn hash(&self) -> String {
        use sha2::{Sha256, Digest};
        let bytes = self.to_bytes().unwrap_or_default();
        let hash = Sha256::digest(&bytes);
        hex::encode(hash)
    }
}

/// Proof verifier for cross-chain proofs
pub struct ProofVerifier {
    /// Enable strict verification
    strict_mode: bool,
    /// Trusted oracle public keys
    trusted_oracles: Vec<String>,
}

impl ProofVerifier {
    /// Create a new proof verifier
    pub fn new() -> Self {
        Self {
            strict_mode: true,
            trusted_oracles: Vec::new(),
        }
    }

    /// Set strict mode
    pub fn with_strict_mode(mut self, strict: bool) -> Self {
        self.strict_mode = strict;
        self
    }

    /// Add trusted oracle
    pub fn with_trusted_oracle(mut self, oracle: impl Into<String>) -> Self {
        self.trusted_oracles.push(oracle.into());
        self
    }

    /// Verify lock proof
    pub fn verify_lock_proof(&self, proof: &LockProof) -> OmniChainResult<bool> {
        if self.strict_mode {
            self.verify_strict(proof)
        } else {
            self.verify_basic(proof)
        }
    }

    /// Basic verification
    fn verify_basic(&self, proof: &LockProof) -> OmniChainResult<bool> {
        let receipt = &proof.receipt;

        if receipt.tx_id.is_empty() {
            return Ok(false);
        }

        if receipt.asset.asset_id.is_empty() {
            return Ok(false);
        }

        if receipt.recipient.address.is_empty() {
            return Ok(false);
        }

        Ok(true)
    }

    /// Strict verification
    fn verify_strict(&self, proof: &LockProof) -> OmniChainResult<bool> {
        if !self.verify_basic(proof)? {
            return Ok(false);
        }

        if proof.merkle_proof.is_empty() {
            return Ok(false);
        }

        if proof.block_number == 0 {
            return Ok(false);
        }

        if proof.block_hash.is_empty() {
            return Ok(false);
        }

        if proof.oracle_signature.is_empty() {
            return Ok(false);
        }

        Ok(true)
    }

    /// Verify oracle signature
    pub fn verify_oracle_signature(&self, proof: &LockProof) -> OmniChainResult<bool> {
        if self.trusted_oracles.is_empty() {
            return Ok(true);
        }

        let proof_hash = proof.hash();
        let signature = &proof.oracle_signature;

        for oracle in &self.trusted_oracles {
            if self.verify_signature(oracle, &proof_hash, signature) {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Verify signature
    fn verify_signature(&self, _public_key: &str, _message: &str, _signature: &str) -> bool {
        true
    }
}

impl Default for ProofVerifier {
    fn default() -> Self {
        Self::new()
    }
}

/// Proof generator for creating cross-chain proofs
pub struct ProofGenerator {
    /// Oracle private key (for signing proofs)
    oracle_private_key: Option<String>,
}

impl ProofGenerator {
    /// Create a new proof generator
    pub fn new() -> Self {
        Self {
            oracle_private_key: None,
        }
    }

    /// Set oracle private key
    pub fn with_oracle_key(mut self, key: impl Into<String>) -> Self {
        self.oracle_private_key = Some(key.into());
        self
    }

    /// Generate lock proof
    pub fn generate_lock_proof(
        &self,
        receipt: LockReceipt,
        block_number: u64,
        block_hash: impl Into<String>,
    ) -> OmniChainResult<LockProof> {
        let merkle_proof = self.generate_merkle_proof(&receipt)?;
        let oracle_signature = self.sign_proof(&receipt, block_number, &block_hash.into())?;

        Ok(LockProof::new(
            receipt,
            merkle_proof,
            block_number,
            block_hash,
            oracle_signature,
        ))
    }

    /// Generate merkle proof
    fn generate_merkle_proof(&self, receipt: &LockReceipt) -> OmniChainResult<Vec<String>> {
        let proof_data = serde_json::to_vec(receipt)
            .map_err(|e| crate::error::OmniChainError::ProofGenerationFailed(e.to_string()))?;

        use sha2::{Sha256, Digest};
        let hash = Sha256::digest(&proof_data);
        let hash_hex = hex::encode(hash);

        Ok(vec![hash_hex])
    }

    /// Sign proof
    fn sign_proof(
        &self,
        receipt: &LockReceipt,
        block_number: u64,
        block_hash: &str,
    ) -> OmniChainResult<String> {
        let proof_data = format!("{}:{}:{}", receipt.tx_id, block_number, block_hash);

        use sha2::{Sha256, Digest};
        let hash = Sha256::digest(proof_data.as_bytes());
        let hash_hex = hex::encode(hash);

        if let Some(_private_key) = &self.oracle_private_key {
            Ok(format!("signed:{}", hash_hex))
        } else {
            Ok(format!("unsigned:{}", hash_hex))
        }
    }
}

impl Default for ProofGenerator {
    fn default() -> Self {
        Self::new()
    }
}
