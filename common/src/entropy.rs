use serde::{Deserialize, Serialize};

/// Entropy measurement for a system state
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EntropyMeasurement {
    /// Entropy in joules per kelvin
    pub value: f64,
    /// Confidence level (0.0 to 1.0)
    pub confidence: f64,
    /// Domain-specific context
    pub domain: String,
    /// Measurement methodology
    pub method: MeasurementMethod,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum MeasurementMethod {
    /// Direct physical measurement
    Physical { sensor_id: String },
    /// Information-theoretic calculation
    Information { bits: u64 },
    /// Economic/market-based proxy
    Economic { market_depth: f64 },
    /// Social/network complexity
    Social { network_size: u64, connectivity: f64 },
    /// Computational complexity reduction
    Computational { operations_saved: u64 },
    /// Custom domain-specific method
    Custom { name: String, params: Vec<f64> },
}

/// Entropy reduction calculation
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EntropyReduction {
    /// Initial state entropy
    pub initial: EntropyMeasurement,
    /// Final state entropy
    pub final_state: EntropyMeasurement,
    /// Net reduction (positive = order created)
    pub delta: f64,
    /// Time taken for the reduction
    pub duration_ns: u64,
    /// Validators who confirmed this reduction
    pub validators: Vec<ValidatorAttestation>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ValidatorAttestation {
    pub validator_id: crate::Address,
    pub reputation_score: f64,
    pub measurement: EntropyMeasurement,
    pub signature: crate::Signature,
}

impl EntropyReduction {
    pub fn calculate_xp(&self) -> u128 {
        // XP = ΔS / c_L²
        // Where c_L is causal loop closure speed
        
        if self.delta <= 0.0 {
            return 0;
        }
        
        // Calculate weighted confidence from validators
        let total_reputation: f64 = self.validators.iter()
            .map(|v| v.reputation_score)
            .sum();
        
        let weighted_confidence: f64 = self.validators.iter()
            .map(|v| v.measurement.confidence * (v.reputation_score / total_reputation))
            .sum();
        
        // Causal loop closure speed factor (simplified)
        let closure_speed = (self.duration_ns as f64) / 1_000_000_000.0; // Convert to seconds
        let c_l_squared = closure_speed.max(0.001).powi(2);
        
        // Base XP calculation
        let base_xp = (self.delta / c_l_squared) * weighted_confidence;
        
        // Scale to integer XP tokens (10^18 precision)
        (base_xp * 1e18) as u128
    }
    
    pub fn verify_validators_consensus(&self) -> bool {
        // Check that validators agree within reasonable bounds
        let measurements: Vec<f64> = self.validators.iter()
            .map(|v| v.measurement.value)
            .collect();
        
        if measurements.is_empty() {
            return false;
        }
        
        let mean = measurements.iter().sum::<f64>() / measurements.len() as f64;
        let variance = measurements.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / measurements.len() as f64;
        
        let std_dev = variance.sqrt();
        
        // Require measurements to be within 2 standard deviations
        measurements.iter().all(|&x| (x - mean).abs() <= 2.0 * std_dev)
    }
}