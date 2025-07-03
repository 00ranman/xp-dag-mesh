use common::{Hash, Address, Result, Error, EntropyReduction};
use crate::validator::{ValidationResponse, ValidationResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Consensus mechanism for validator responses
pub trait ConsensusMechanism: Send + Sync {
    /// Aggregate validation responses into a consensus result
    fn aggregate_validations(
        &self,
        responses: Vec<ValidationResponse>,
        validator_weights: &HashMap<Address, f64>,
    ) -> Result<ConsensusResult>;
}

/// Result of consensus aggregation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConsensusResult {
    pub request_id: Hash,
    pub is_valid: bool,
    pub confidence: f64,
    pub entropy_reduction: Option<EntropyReduction>,
    pub participating_validators: Vec<Address>,
    pub dissenting_validators: Vec<Address>,
    pub consensus_type: ConsensusType,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ConsensusType {
    /// Unanimous agreement
    Unanimous,
    /// Weighted majority (>66.7%)
    WeightedMajority { percentage: f64 },
    /// Simple majority (>50%)
    SimpleMajority { percentage: f64 },
    /// No consensus reached
    NoConsensus,
}

/// Entropy-weighted consensus implementation
pub struct EntropyWeightedConsensus {
    pub required_confidence: f64,
    pub majority_threshold: f64,
}

impl ConsensusMechanism for EntropyWeightedConsensus {
    fn aggregate_validations(
        &self,
        responses: Vec<ValidationResponse>,
        validator_weights: &HashMap<Address, f64>,
    ) -> Result<ConsensusResult> {
        if responses.is_empty() {
            return Err(Error::Other("No validation responses".into()));
        }

        let request_id = responses[0].request_id;
        
        // Calculate weighted votes
        let mut valid_weight = 0.0;
        let mut invalid_weight = 0.0;
        let mut total_weight = 0.0;
        let mut participating = Vec::new();
        let mut dissenting = Vec::new();
        let mut entropy_measurements = Vec::new();

        for response in &responses {
            let weight = validator_weights.get(&response.validator)
                .copied()
                .unwrap_or(1.0);
            
            total_weight += weight;
            participating.push(response.validator);

            match &response.result {
                ValidationResult::Valid { score } => {
                    valid_weight += weight * score * response.confidence;
                    if let Some(measurement) = &response.entropy_measurement {
                        entropy_measurements.push((measurement.clone(), weight));
                    }
                }
                ValidationResult::Invalid { .. } => {
                    invalid_weight += weight * response.confidence;
                    dissenting.push(response.validator);
                }
                ValidationResult::Uncertain { confidence, .. } => {
                    // Split weight based on uncertainty
                    valid_weight += weight * confidence * 0.5;
                    invalid_weight += weight * (1.0 - confidence) * 0.5;
                }
            }
        }

        // Calculate consensus percentage
        let valid_percentage = valid_weight / total_weight;
        let is_valid = valid_percentage >= self.majority_threshold;
        
        // Determine consensus type
        let consensus_type = if valid_percentage >= 0.999 {
            ConsensusType::Unanimous
        } else if valid_percentage >= 0.667 {
            ConsensusType::WeightedMajority { percentage: valid_percentage }
        } else if valid_percentage >= 0.5 {
            ConsensusType::SimpleMajority { percentage: valid_percentage }
        } else {
            ConsensusType::NoConsensus
        };

        // Aggregate entropy measurements if valid
        let entropy_reduction = if is_valid && !entropy_measurements.is_empty() {
            Some(self.aggregate_entropy_measurements(entropy_measurements))
        } else {
            None
        };

        Ok(ConsensusResult {
            request_id,
            is_valid,
            confidence: valid_percentage,
            entropy_reduction,
            participating_validators: participating,
            dissenting_validators: dissenting,
            consensus_type,
        })
    }
}

impl EntropyWeightedConsensus {
    fn aggregate_entropy_measurements(
        &self,
        measurements: Vec<(common::EntropyMeasurement, f64)>,
    ) -> EntropyReduction {
        // Weighted average of entropy values
        let total_weight: f64 = measurements.iter().map(|(_, w)| w).sum();
        
        let initial_entropy: f64 = measurements.iter()
            .map(|(m, w)| m.value * w)
            .sum::<f64>() / total_weight;
        
        // For now, use the first measurement's metadata
        let first_measurement = &measurements[0].0;
        
        EntropyReduction {
            initial: common::EntropyMeasurement {
                value: initial_entropy,
                confidence: first_measurement.confidence,
                domain: first_measurement.domain.clone(),
                method: first_measurement.method.clone(),
            },
            final_state: first_measurement.clone(),
            delta: initial_entropy - first_measurement.value,
            duration_ns: 0, // Would be set by actual measurement
            validators: vec![], // Would be filled with actual attestations
        }
    }
}

/// Byzantine fault tolerant consensus
pub struct ByzantineConsensus {
    pub byzantine_threshold: f64, // Typically 1/3
}

impl ConsensusMechanism for ByzantineConsensus {
    fn aggregate_validations(
        &self,
        responses: Vec<ValidationResponse>,
        validator_weights: &HashMap<Address, f64>,
    ) -> Result<ConsensusResult> {
        // Implement PBFT-style consensus
        // This is a simplified version - full PBFT would include multiple rounds
        
        let total_validators = responses.len();
        let byzantine_limit = (total_validators as f64 * self.byzantine_threshold) as usize;
        
        // Group responses by result
        let mut result_groups: HashMap<String, Vec<&ValidationResponse>> = HashMap::new();
        
        for response in &responses {
            let key = match &response.result {
                ValidationResult::Valid { score } => format!("valid_{}", score),
                ValidationResult::Invalid { reason } => format!("invalid_{}", reason),
                ValidationResult::Uncertain { .. } => "uncertain".to_string(),
            };
            result_groups.entry(key).or_default().push(response);
        }
        
        // Find the largest group
        let largest_group = result_groups.values()
            .max_by_key(|group| group.len())
            .ok_or_else(|| Error::Other("No consensus group found".into()))?;
        
        // Check if we have enough agreement (2f+1 where f is byzantine limit)
        let required_agreement = total_validators - byzantine_limit;
        
        if largest_group.len() >= required_agreement {
            // We have consensus
            let is_valid = matches!(largest_group[0].result, ValidationResult::Valid { .. });
            
            Ok(ConsensusResult {
                request_id: responses[0].request_id,
                is_valid,
                confidence: largest_group.len() as f64 / total_validators as f64,
                entropy_reduction: None, // Would aggregate from responses
                participating_validators: responses.iter().map(|r| r.validator).collect(),
                dissenting_validators: vec![], // Would calculate dissenters
                consensus_type: ConsensusType::WeightedMajority {
                    percentage: largest_group.len() as f64 / total_validators as f64,
                },
            })
        } else {
            Ok(ConsensusResult {
                request_id: responses[0].request_id,
                is_valid: false,
                confidence: 0.0,
                entropy_reduction: None,
                participating_validators: responses.iter().map(|r| r.validator).collect(),
                dissenting_validators: vec![],
                consensus_type: ConsensusType::NoConsensus,
            })
        }
    }
}