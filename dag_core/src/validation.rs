use crate::transaction::Transaction;
use common::{Hash, Address, Result, Error, XpTime, EntropyReduction};
use serde::{Deserialize, Serialize};

/// Validation engine for DAG transactions with entropy constraints

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransactionValidator {
    /// Entropy validation threshold
    pub entropy_threshold: f64,
    /// Maximum allowed causal closure speed
    pub max_closure_speed: f64,
    /// Reputation threshold for validators
    pub validator_reputation_threshold: f64,
    /// Temporal validation window (minutes)
    pub temporal_window_minutes: u64,
}

impl Default for TransactionValidator {
    fn default() -> Self {
        Self {
            entropy_threshold: 0.1,
            max_closure_speed: 100.0,
            validator_reputation_threshold: 0.5,
            temporal_window_minutes: 1440, // 24 hours
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether transaction is valid
    pub is_valid: bool,
    /// Validation confidence (0.0 to 1.0)
    pub confidence: f64,
    /// List of validation errors
    pub errors: Vec<ValidationError>,
    /// List of validation warnings
    pub warnings: Vec<ValidationWarning>,
    /// Entropy compliance check
    pub entropy_compliant: bool,
    /// Temporal compliance check
    pub temporal_compliant: bool,
    /// Validator who performed validation
    pub validator: Option<Address>,
    /// Validation timestamp
    pub validated_at: XpTime,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ValidationError {
    /// Invalid entropy reduction claim
    InvalidEntropyReduction { claimed: f64, expected: f64 },
    /// Causal closure speed exceeds limits
    ExcessiveCausalSpeed { speed: f64, limit: f64 },
    /// Invalid temporal ordering
    TemporalOrderingViolation { message: String },
    /// Insufficient validator reputation
    InsufficientValidatorReputation { reputation: f64, required: f64 },
    /// Thermodynamic law violation
    ThermodynamicViolation { law: String, details: String },
    /// Invalid signature
    InvalidSignature,
    /// Missing required parents
    MissingParents { expected: Vec<Hash> },
    /// Cyclic dependency detected
    CyclicDependency { cycle: Vec<Hash> },
    /// Insufficient balance for transaction
    InsufficientBalance { required: u128, available: u128 },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ValidationWarning {
    /// Low entropy reduction efficiency
    LowEntropyEfficiency { efficiency: f64 },
    /// Temporal drift detected
    TemporalDrift { drift_minutes: i64 },
    /// Unusual causal patterns
    UnusualCausalPattern { description: String },
    /// High energy consumption
    HighEnergyConsumption { consumption: f64 },
    /// Potential gaming attempt
    PotentialGaming { indicators: Vec<String> },
}

impl TransactionValidator {
    /// Validate a transaction comprehensively
    pub fn validate(&self, transaction: &Transaction) -> Result<ValidationResult> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let validation_time = XpTime::now();

        // Basic structural validation
        self.validate_structure(transaction, &mut errors)?;

        // Entropy validation
        let entropy_compliant = self.validate_entropy(transaction, &mut errors, &mut warnings)?;

        // Temporal validation
        let temporal_compliant = self.validate_temporal(transaction, &mut errors, &mut warnings)?;

        // Thermodynamic validation
        self.validate_thermodynamics(transaction, &mut errors)?;

        // Signature validation
        self.validate_signature(transaction, &mut errors)?;

        // Causal ordering validation
        self.validate_causal_ordering(transaction, &mut errors)?;

        // Gaming detection
        self.detect_gaming_attempts(transaction, &mut warnings)?;

        let is_valid = errors.is_empty();
        let confidence = self.calculate_confidence(&errors, &warnings);

        Ok(ValidationResult {
            is_valid,
            confidence,
            errors,
            warnings,
            entropy_compliant,
            temporal_compliant,
            validator: None, // Would be set by calling validator
            validated_at: validation_time,
        })
    }

    /// Validate transaction structure
    fn validate_structure(&self, transaction: &Transaction, errors: &mut Vec<ValidationError>) -> Result<()> {
        // Check that hash is correctly computed
        let computed_hash = transaction.compute_hash();
        if transaction.hash != computed_hash {
            errors.push(ValidationError::InvalidSignature);
        }

        // Check nonce ordering (simplified - would need DAG context for full validation)
        if transaction.nonce == u64::MAX {
            errors.push(ValidationError::TemporalOrderingViolation { 
                message: "Nonce overflow".to_string() 
            });
        }

        Ok(())
    }

    /// Validate entropy constraints
    fn validate_entropy(
        &self, 
        transaction: &Transaction, 
        errors: &mut Vec<ValidationError>,
        warnings: &mut Vec<ValidationWarning>,
    ) -> Result<bool> {
        use crate::transaction::TransactionType;

        match &transaction.tx_type {
            TransactionType::EntropyReductionClaim { reduction, .. } => {
                // Check entropy reduction is positive and above threshold
                if reduction.delta <= 0.0 {
                    errors.push(ValidationError::InvalidEntropyReduction {
                        claimed: reduction.delta,
                        expected: self.entropy_threshold,
                    });
                    return Ok(false);
                }

                if reduction.delta < self.entropy_threshold {
                    warnings.push(ValidationWarning::LowEntropyEfficiency {
                        efficiency: reduction.delta / self.entropy_threshold,
                    });
                }

                // Validate entropy measurement methodology
                if !reduction.verify_validators_consensus() {
                    errors.push(ValidationError::ThermodynamicViolation {
                        law: "Measurement Consensus".to_string(),
                        details: "Validator measurements do not agree".to_string(),
                    });
                    return Ok(false);
                }
            }

            TransactionType::PhysicsXpMint { calculation_input, .. } => {
                // Validate physics calculation inputs
                if calculation_input.entropy_reduction <= 0.0 {
                    errors.push(ValidationError::InvalidEntropyReduction {
                        claimed: calculation_input.entropy_reduction,
                        expected: self.entropy_threshold,
                    });
                    return Ok(false);
                }

                // Check causal closure speed
                if calculation_input.closure_speed > self.max_closure_speed {
                    errors.push(ValidationError::ExcessiveCausalSpeed {
                        speed: calculation_input.closure_speed,
                        limit: self.max_closure_speed,
                    });
                    return Ok(false);
                }

                // Validate reputation bounds
                if calculation_input.reputation < 0.0 || calculation_input.reputation > 1.0 {
                    errors.push(ValidationError::InsufficientValidatorReputation {
                        reputation: calculation_input.reputation,
                        required: 0.0,
                    });
                    return Ok(false);
                }
            }

            TransactionType::TemporalLoop { temporal_loop, .. } => {
                // Validate temporal loop constraints
                if let Some(final_entropy) = temporal_loop.final_entropy {
                    let reduction = temporal_loop.initial_entropy - final_entropy;
                    if reduction <= 0.0 {
                        errors.push(ValidationError::InvalidEntropyReduction {
                            claimed: reduction,
                            expected: self.entropy_threshold,
                        });
                        return Ok(false);
                    }
                }
            }

            _ => {
                // Other transaction types don't have entropy constraints
            }
        }

        Ok(true)
    }

    /// Validate temporal constraints
    fn validate_temporal(
        &self,
        transaction: &Transaction,
        errors: &mut Vec<ValidationError>,
        warnings: &mut Vec<ValidationWarning>,
    ) -> Result<bool> {
        let current_time = XpTime::now();
        
        // Check if transaction is from the future (with small tolerance)
        let tx_time = transaction.xp_time;
        if tx_time.to_total_minutes() > current_time.to_total_minutes() + 10 {
            errors.push(ValidationError::TemporalOrderingViolation {
                message: "Transaction from future".to_string(),
            });
            return Ok(false);
        }

        // Check if transaction is too old
        let age_minutes = current_time.to_total_minutes() - tx_time.to_total_minutes();
        if age_minutes > self.temporal_window_minutes {
            errors.push(ValidationError::TemporalOrderingViolation {
                message: "Transaction too old".to_string(),
            });
            return Ok(false);
        }

        // Convert to Gregorian and check for drift
        let gregorian_timestamp = tx_time.to_gregorian_timestamp();
        let current_gregorian = current_time.to_gregorian_timestamp();
        let drift_seconds = (current_gregorian as i64) - (gregorian_timestamp as i64);
        
        if drift_seconds.abs() > 300 { // 5 minute tolerance
            warnings.push(ValidationWarning::TemporalDrift {
                drift_minutes: drift_seconds / 60,
            });
        }

        Ok(true)
    }

    /// Validate thermodynamic constraints
    fn validate_thermodynamics(&self, transaction: &Transaction, errors: &mut Vec<ValidationError>) -> Result<()> {
        use crate::transaction::TransactionType;

        match &transaction.tx_type {
            TransactionType::PhysicsXpMint { calculation_input, .. } => {
                // Check energy conservation
                if let Some(power) = calculation_input.power_consumption {
                    if power < 0.0 {
                        errors.push(ValidationError::ThermodynamicViolation {
                            law: "Energy Conservation".to_string(),
                            details: "Negative power consumption".to_string(),
                        });
                    }
                }

                // Check temperature constraint
                if let Some(temp) = calculation_input.temperature {
                    if temp <= 0.0 {
                        errors.push(ValidationError::ThermodynamicViolation {
                            law: "Temperature Constraint".to_string(),
                            details: "Temperature must be positive (Kelvin)".to_string(),
                        });
                    }
                }

                // Check entropy constraint (second law)
                if calculation_input.entropy_reduction <= 0.0 {
                    errors.push(ValidationError::ThermodynamicViolation {
                        law: "Second Law of Thermodynamics".to_string(),
                        details: "Local entropy reduction requires external work".to_string(),
                    });
                }
            }
            _ => {
                // Other transactions don't have specific thermodynamic constraints
            }
        }

        Ok(())
    }

    /// Validate transaction signature
    fn validate_signature(&self, transaction: &Transaction, errors: &mut Vec<ValidationError>) -> Result<()> {
        // Note: In a real implementation, we would need the public key
        // For now, just check that signature is not empty
        if transaction.signature.as_bytes().is_empty() {
            errors.push(ValidationError::InvalidSignature);
        }

        Ok(())
    }

    /// Validate causal ordering
    fn validate_causal_ordering(&self, transaction: &Transaction, errors: &mut Vec<ValidationError>) -> Result<()> {
        // Check parent references make sense
        if transaction.parents.is_empty() {
            // Genesis transactions should have no parents, others should
            // This would need DAG context to validate properly
        }

        // Check for self-references
        if transaction.parents.contains(&transaction.hash) {
            errors.push(ValidationError::CyclicDependency {
                cycle: vec![transaction.hash],
            });
        }

        Ok(())
    }

    /// Detect potential gaming attempts
    fn detect_gaming_attempts(&self, transaction: &Transaction, warnings: &mut Vec<ValidationWarning>) -> Result<()> {
        let mut indicators = Vec::new();

        // Check for unusually high entropy claims
        if let crate::transaction::TransactionType::EntropyReductionClaim { reduction, .. } = &transaction.tx_type {
            if reduction.delta > 10.0 { // Arbitrarily high threshold
                indicators.push("Unusually high entropy reduction claim".to_string());
            }
        }

        // Check for suspicious timing patterns
        let tx_minute = transaction.xp_time.minute;
        if tx_minute == 0 || tx_minute == 50 || tx_minute == 99 {
            indicators.push("Suspicious timing pattern".to_string());
        }

        // Check for round number nonces (potential automation)
        if transaction.nonce % 100 == 0 && transaction.nonce > 0 {
            indicators.push("Round number nonce pattern".to_string());
        }

        if !indicators.is_empty() {
            warnings.push(ValidationWarning::PotentialGaming { indicators });
        }

        Ok(())
    }

    /// Calculate validation confidence
    fn calculate_confidence(&self, errors: &[ValidationError], warnings: &[ValidationWarning]) -> f64 {
        if !errors.is_empty() {
            return 0.0;
        }

        let base_confidence = 1.0;
        let warning_penalty = warnings.len() as f64 * 0.1;
        
        (base_confidence - warning_penalty).max(0.1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transaction::{Transaction, TransactionType, TransactionMetadata};
    use common::{XpCalculationInput, TemporalLoop, LoopType};

    #[test]
    fn test_valid_transaction_validation() {
        let validator = TransactionValidator::default();
        
        let tx = Transaction::new(
            TransactionType::Transfer {
                from: Address::new([1; 20]),
                to: Address::new([2; 20]),
                amount: common::XpAmount::new(1000, 0.5),
            },
            vec![],
            1,
            TransactionMetadata::default(),
        );

        let result = validator.validate(&tx).unwrap();
        assert!(result.is_valid);
        assert!(result.confidence > 0.5);
    }

    #[test]
    fn test_invalid_entropy_transaction() {
        let validator = TransactionValidator::default();
        
        let invalid_reduction = common::EntropyReduction {
            initial: common::EntropyMeasurement {
                value: 5.0,
                confidence: 0.9,
                domain: "test".to_string(),
                method: common::MeasurementMethod::Information { bits: 100 },
            },
            final_state: common::EntropyMeasurement {
                value: 6.0, // Higher than initial - invalid
                confidence: 0.9,
                domain: "test".to_string(),
                method: common::MeasurementMethod::Information { bits: 120 },
            },
            delta: -1.0, // Negative delta = entropy increase
            duration_ns: 1000000,
            validators: vec![],
        };

        let tx = Transaction::new(
            TransactionType::EntropyReductionClaim {
                claimant: Address::new([1; 20]),
                reduction: invalid_reduction,
            },
            vec![],
            1,
            TransactionMetadata::default(),
        );

        let result = validator.validate(&tx).unwrap();
        assert!(!result.is_valid);
        assert!(!result.entropy_compliant);
    }
}