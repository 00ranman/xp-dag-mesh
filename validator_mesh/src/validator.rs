use common::{Address, Hash, PublicKey, Signature, XpAmount, EntropyMeasurement, Result, Error};
use dag_core::Transaction;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use async_trait::async_trait;

/// Validator node in the mesh
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Validator {
    /// Validator's address
    pub address: Address,
    /// Public key for signature verification
    pub public_key: PublicKey,
    /// Current stake amount
    pub stake: XpAmount,
    /// Reputation score (0.0 to 1.0)
    pub reputation: f64,
    /// Total entropy reduced by this validator
    pub total_entropy_reduced: f64,
    /// Specialization domains
    pub specializations: Vec<ValidatorDomain>,
    /// Performance metrics
    pub metrics: ValidatorMetrics,
    /// Current status
    pub status: ValidatorStatus,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ValidatorDomain {
    /// Physical measurements (sensors, IoT)
    Physical { sensor_types: Vec<String> },
    /// Information theory calculations
    Information { data_types: Vec<String> },
    /// Economic/market analysis
    Economic { markets: Vec<String> },
    /// Social network analysis
    Social { platforms: Vec<String> },
    /// Computational optimization
    Computational { languages: Vec<String> },
    /// Smart contract validation
    Contracts { vm_types: Vec<String> },
    /// General purpose
    General,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ValidatorMetrics {
    /// Total validations performed
    pub total_validations: u64,
    /// Successful validations
    pub successful_validations: u64,
    /// Average validation time in ms
    pub avg_validation_time_ms: f64,
    /// Entropy measurements accuracy
    pub measurement_accuracy: f64,
    /// Last active timestamp
    pub last_active: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ValidatorStatus {
    Active,
    Inactive,
    Suspended { until: u64, reason: String },
    Slashed { amount: XpAmount, reason: String },
}

/// Validation request sent to validators
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ValidationRequest {
    pub request_id: Hash,
    pub transaction: Transaction,
    pub validation_type: ValidationType,
    pub deadline: u64,
    pub min_validators: u32,
    pub requestor: Address,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ValidationType {
    /// Validate entropy reduction claim
    EntropyReduction,
    /// Validate smart contract execution
    ContractExecution,
    /// Validate loop closure
    LoopClosure,
    /// General transaction validation
    Transaction,
}

/// Validation response from a validator
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ValidationResponse {
    pub request_id: Hash,
    pub validator: Address,
    pub result: ValidationResult,
    pub entropy_measurement: Option<EntropyMeasurement>,
    pub confidence: f64,
    pub signature: Signature,
    pub timestamp: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ValidationResult {
    Valid { score: f64 },
    Invalid { reason: String },
    Uncertain { confidence: f64, notes: String },
}

/// Validator behavior trait
#[async_trait]
pub trait ValidatorNode: Send + Sync {
    /// Process a validation request
    async fn validate(&self, request: ValidationRequest) -> Result<ValidationResponse>;
    
    /// Get validator info
    fn info(&self) -> &Validator;
    
    /// Update reputation based on validation outcomes
    fn update_reputation(&mut self, delta: f64);
    
    /// Check if validator can handle a specific domain
    fn can_validate_domain(&self, domain: &ValidatorDomain) -> bool;
    
    /// Calculate validation weight based on stake and reputation
    fn validation_weight(&self) -> f64 {
        let stake_weight = (self.info().stake.amount as f64 / 1e18).sqrt();
        let reputation_weight = self.info().reputation;
        stake_weight * reputation_weight
    }
}

/// Validator selection strategy
pub trait ValidatorSelector: Send + Sync {
    /// Select validators for a specific validation task
    fn select_validators(
        &self,
        available: &[Address],
        request: &ValidationRequest,
        validator_info: &HashMap<Address, Validator>,
    ) -> Vec<Address>;
}

/// Entropy-weighted validator selection
pub struct EntropyWeightedSelector {
    pub min_validators: usize,
    pub max_validators: usize,
    pub domain_weight: f64,
    pub reputation_threshold: f64,
}

impl ValidatorSelector for EntropyWeightedSelector {
    fn select_validators(
        &self,
        available: &[Address],
        request: &ValidationRequest,
        validator_info: &HashMap<Address, Validator>,
    ) -> Vec<Address> {
        let mut candidates: Vec<(Address, f64)> = available
            .iter()
            .filter_map(|addr| {
                let validator = validator_info.get(addr)?;
                
                // Filter by reputation threshold
                if validator.reputation < self.reputation_threshold {
                    return None;
                }
                
                // Filter by status
                if validator.status != ValidatorStatus::Active {
                    return None;
                }
                
                // Calculate selection score
                let base_weight = validator.validation_weight();
                let domain_bonus = self.calculate_domain_bonus(validator, request);
                let entropy_bonus = validator.total_entropy_reduced.log2().max(1.0);
                
                let score = base_weight * (1.0 + domain_bonus * self.domain_weight) * entropy_bonus;
                
                Some((*addr, score))
            })
            .collect();
        
        // Sort by score (highest first)
        candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        // Select top validators
        let count = self.min_validators.max(request.min_validators as usize)
            .min(self.max_validators)
            .min(candidates.len());
        
        candidates.into_iter()
            .take(count)
            .map(|(addr, _)| addr)
            .collect()
    }
}

impl EntropyWeightedSelector {
    fn calculate_domain_bonus(&self, validator: &Validator, request: &ValidationRequest) -> f64 {
        match request.validation_type {
            ValidationType::EntropyReduction => {
                validator.specializations.iter()
                    .filter(|s| matches!(s, ValidatorDomain::Physical { .. } | ValidatorDomain::Information { .. }))
                    .count() as f64
            }
            ValidationType::ContractExecution => {
                validator.specializations.iter()
                    .filter(|s| matches!(s, ValidatorDomain::Contracts { .. } | ValidatorDomain::Computational { .. }))
                    .count() as f64
            }
            _ => 0.0,
        }
    }
}