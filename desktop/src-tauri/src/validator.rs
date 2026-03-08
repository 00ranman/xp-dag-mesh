//! DAG validation and consensus for mesh network
//! Proof-of-work/stake hybrid with tiered access verification

use crate::wallet::{AccessTier, Wallet};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// A vertex in the DAG
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DagVertex {
    pub id: String,
    pub parent_ids: Vec<String>,
    pub payload: Vec<u8>,
    pub timestamp: u64,
    pub proposer: String,
    pub signature: Vec<u8>,
    pub nonce: u64,
    pub difficulty: u32,
}

/// Validation result
#[derive(Debug, Clone)]
pub enum ValidationResult {
    Valid,
    InvalidSignature,
    InsufficientTier,
    InvalidPoW,
    OrphanParents(Vec<String>),
    TimestampDrift,
}

/// Hybrid PoW/PoS validator
pub struct Validator {
    pub min_difficulty: u32,
    pub max_timestamp_drift_secs: u64,
    pub required_confirmations: usize,
}

impl Default for Validator {
    fn default() -> Self {
        Self {
            min_difficulty: 16,
            max_timestamp_drift_secs: 300,
            required_confirmations: 3,
        }
    }
}

impl Validator {
    /// Validate a vertex against the DAG rules
    pub fn validate(
        &self,
        vertex: &DagVertex,
        proposer_wallet: &Wallet,
        known_ids: &[String],
    ) -> ValidationResult {
        // 1. Tier check: must be Contributor or above to propose
        if !proposer_wallet.can_perform(&AccessTier::Contributor) {
            return ValidationResult::InsufficientTier;
        }

        // 2. Timestamp drift check
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        if vertex.timestamp > now + self.max_timestamp_drift_secs
            || vertex.timestamp + self.max_timestamp_drift_secs < now
        {
            return ValidationResult::TimestampDrift;
        }

        // 3. Parent existence check
        let orphans: Vec<String> = vertex
            .parent_ids
            .iter()
            .filter(|pid| !known_ids.contains(pid))
            .cloned()
            .collect();
        if !orphans.is_empty() {
            return ValidationResult::OrphanParents(orphans);
        }

        // 4. Proof-of-work check (leading zero bits)
        if !self.verify_pow(vertex) {
            return ValidationResult::InvalidPoW;
        }

        ValidationResult::Valid
    }

    /// Verify proof-of-work: hash must have min_difficulty leading zero bits
    fn verify_pow(&self, vertex: &DagVertex) -> bool {
        // Placeholder: real impl uses blake3 or sha256
        // For scaffold, check nonce is non-zero when difficulty > 0
        if self.min_difficulty == 0 {
            return true;
        }
        vertex.nonce > 0
    }

    /// Adaptive difficulty based on proposer tier
    /// Higher tiers get reduced PoW requirements (PoS benefit)
    pub fn effective_difficulty(&self, wallet: &Wallet) -> u32 {
        match wallet.tier {
            AccessTier::Coordinator => self.min_difficulty / 4,
            AccessTier::Validator => self.min_difficulty / 2,
            AccessTier::Contributor => self.min_difficulty,
            AccessTier::Observer => self.min_difficulty * 2,
        }
    }
}
