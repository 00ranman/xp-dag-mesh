use serde::{Deserialize, Serialize};
use crate::entropy::EntropyReduction;
use crate::temporal::XpTime;
use crate::types::{Address, Result, Error};

/// Enhanced XP calculation engine implementing the full physics formula:
/// XP = R × F × ΔS × (w · E) × log(1/Tₛ)

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PhysicsXpCalculator {
    /// Domain weighting vectors for different contexts
    pub domain_weights: DomainWeights,
    /// Thermodynamic constants
    pub constants: ThermodynamicConstants,
    /// Calibration parameters
    pub calibration: CalibrationParameters,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DomainWeights {
    /// Cognitive domain weight
    pub cognitive: f64,
    /// Physical domain weight  
    pub physical: f64,
    /// Social domain weight
    pub social: f64,
    /// Economic domain weight
    pub economic: f64,
    /// Creative domain weight
    pub creative: f64,
    /// System domain weight
    pub system: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ThermodynamicConstants {
    /// Boltzmann constant equivalent for XP system
    pub xp_boltzmann: f64,
    /// Energy scale factor
    pub energy_scale: f64,
    /// Temperature reference (Kelvin)
    pub temp_reference: f64,
    /// Causal closure speed constant
    pub closure_speed_constant: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CalibrationParameters {
    /// Base scaling factor
    pub base_scale: f64,
    /// Reputation amplification factor
    pub reputation_amplification: f64,
    /// Temporal sustainability weight
    pub temporal_weight: f64,
    /// Cross-domain bonus multiplier
    pub cross_domain_bonus: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct XpCalculationInput {
    /// Reputation score of the agent (0.0 to 1.0)
    pub reputation: f64,
    /// Feedback loop closure coefficient (0.0 to 1.0)
    pub feedback_closure: f64,
    /// Entropy reduction achieved (in joules/kelvin)
    pub entropy_reduction: f64,
    /// Domain weighting vector
    pub domain_vector: Vec<f64>,
    /// Essentiality vector (criticality to system function)
    pub essentiality_vector: Vec<f64>,
    /// Temporal sustainability measure
    pub temporal_sustainability: f64,
    /// Causal closure speed (events per second)
    pub closure_speed: f64,
    /// Power consumption (watts)
    pub power_consumption: Option<f64>,
    /// Temperature factor (kelvin)
    pub temperature: Option<f64>,
    /// Network impact factor
    pub network_impact: f64,
    /// Collaboration factor
    pub collaboration_factor: f64,
    /// Time of calculation
    pub calculation_time: XpTime,
    /// Associated validator
    pub validator: Option<Address>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct XpCalculationResult {
    /// Final XP amount
    pub xp_amount: u128,
    /// Individual formula components
    pub components: XpComponents,
    /// Calculation metadata
    pub metadata: XpCalculationMetadata,
    /// Thermodynamic compliance
    pub thermodynamic_compliance: ThermodynamicCompliance,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct XpComponents {
    /// Reputation factor (R)
    pub reputation_factor: f64,
    /// Feedback factor (F)
    pub feedback_factor: f64,
    /// Entropy reduction (ΔS)
    pub entropy_delta: f64,
    /// Domain-essentiality product (w · E)
    pub domain_essentiality: f64,
    /// Temporal sustainability factor log(1/Tₛ)
    pub temporal_factor: f64,
    /// Base calculation before scaling
    pub base_value: f64,
    /// Final scaled value
    pub scaled_value: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct XpCalculationMetadata {
    /// Formula type used
    pub formula_type: XpFormulaType,
    /// Calculation timestamp
    pub calculated_at: XpTime,
    /// Confidence level (0.0 to 1.0)
    pub confidence: f64,
    /// Domain context
    pub domain_context: String,
    /// Validator who performed calculation
    pub validator: Option<Address>,
    /// Cross-domain synergies detected
    pub cross_domain_synergies: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum XpFormulaType {
    /// Complete physics formula with all factors
    Complete,
    /// Simplified formula for basic calculations
    Simplified,
    /// Domain-specific calibrated formula
    DomainCalibrated { domain: String },
    /// Emergency/fallback calculation
    Fallback,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ThermodynamicCompliance {
    /// Whether calculation obeys thermodynamic laws
    pub is_compliant: bool,
    /// Energy conservation check
    pub energy_conserved: bool,
    /// Entropy constraint satisfaction
    pub entropy_constraint_satisfied: bool,
    /// Causal ordering preserved
    pub causal_ordering_preserved: bool,
    /// Compliance violations if any
    pub violations: Vec<String>,
    /// Confidence in compliance assessment
    pub compliance_confidence: f64,
}

impl Default for PhysicsXpCalculator {
    fn default() -> Self {
        Self {
            domain_weights: DomainWeights {
                cognitive: 1.2,
                physical: 1.0,
                social: 0.9,
                economic: 1.1,
                creative: 1.15,
                system: 1.3,
            },
            constants: ThermodynamicConstants {
                xp_boltzmann: 1.380649e-23,
                energy_scale: 1e6,
                temp_reference: 298.15, // Room temperature
                closure_speed_constant: 1.0,
            },
            calibration: CalibrationParameters {
                base_scale: 1000.0,
                reputation_amplification: 2.0,
                temporal_weight: 0.5,
                cross_domain_bonus: 1.25,
            },
        }
    }
}

impl PhysicsXpCalculator {
    /// Calculate XP using the complete physics formula
    pub fn calculate_xp(&self, input: &XpCalculationInput) -> Result<XpCalculationResult> {
        // Validate inputs
        self.validate_input(input)?;

        // Calculate individual components
        let components = self.calculate_components(input)?;
        
        // Apply thermodynamic constraints
        let thermodynamic_compliance = self.check_thermodynamic_compliance(input, &components)?;
        
        if !thermodynamic_compliance.is_compliant {
            return Err(Error::Other("Thermodynamic compliance violation".into()));
        }

        // Calculate final XP amount (scaled to integer tokens)
        let xp_amount = (components.scaled_value * 1e18) as u128;

        Ok(XpCalculationResult {
            xp_amount,
            components,
            metadata: XpCalculationMetadata {
                formula_type: XpFormulaType::Complete,
                calculated_at: input.calculation_time,
                confidence: self.calculate_confidence(input),
                domain_context: self.determine_domain_context(input),
                validator: input.validator,
                cross_domain_synergies: self.detect_cross_domain_synergies(input),
            },
            thermodynamic_compliance,
        })
    }

    /// Calculate individual formula components
    fn calculate_components(&self, input: &XpCalculationInput) -> Result<XpComponents> {
        // R: Reputation factor with amplification
        let reputation_factor = input.reputation.powf(self.calibration.reputation_amplification);

        // F: Feedback closure coefficient
        let feedback_factor = input.feedback_closure;

        // ΔS: Entropy reduction (must be positive)
        if input.entropy_reduction <= 0.0 {
            return Err(Error::Other("Entropy reduction must be positive".into()));
        }
        let entropy_delta = input.entropy_reduction;

        // w · E: Domain weighting dot product with essentiality
        let domain_essentiality = self.calculate_domain_essentiality_product(input)?;

        // log(1/Tₛ): Temporal sustainability factor
        let temporal_factor = if input.temporal_sustainability > 0.0 {
            (1.0 / input.temporal_sustainability).ln().max(0.1)
        } else {
            0.1 // Minimum value for stability
        };

        // Causal closure speed factor (c_L²)
        let closure_speed_squared = (input.closure_speed * self.constants.closure_speed_constant).powi(2);
        if closure_speed_squared <= 0.0 {
            return Err(Error::Other("Closure speed must be positive".into()));
        }

        // Base calculation: R × F × (ΔS / c_L²) × (w · E) × log(1/Tₛ)
        let base_value = reputation_factor 
            * feedback_factor 
            * (entropy_delta / closure_speed_squared) 
            * domain_essentiality 
            * temporal_factor;

        // Apply scaling and bonuses
        let mut scaled_value = base_value * self.calibration.base_scale;

        // Network impact bonus
        scaled_value *= input.network_impact;

        // Collaboration bonus
        scaled_value *= input.collaboration_factor;

        // Cross-domain bonus
        if self.detect_cross_domain_synergies(input).len() > 1 {
            scaled_value *= self.calibration.cross_domain_bonus;
        }

        Ok(XpComponents {
            reputation_factor,
            feedback_factor,
            entropy_delta,
            domain_essentiality,
            temporal_factor,
            base_value,
            scaled_value,
        })
    }

    /// Calculate domain-essentiality dot product
    fn calculate_domain_essentiality_product(&self, input: &XpCalculationInput) -> Result<f64> {
        if input.domain_vector.len() != input.essentiality_vector.len() {
            return Err(Error::Other("Domain and essentiality vectors must have same length".into()));
        }

        let dot_product: f64 = input.domain_vector.iter()
            .zip(input.essentiality_vector.iter())
            .map(|(w, e)| w * e)
            .sum();

        Ok(dot_product.max(0.1)) // Ensure positive
    }

    /// Check thermodynamic compliance
    fn check_thermodynamic_compliance(
        &self,
        input: &XpCalculationInput,
        components: &XpComponents,
    ) -> Result<ThermodynamicCompliance> {
        let mut violations = Vec::new();
        let mut energy_conserved = true;
        let mut entropy_constraint_satisfied = true;
        let mut causal_ordering_preserved = true;

        // Energy conservation check
        if let Some(power) = input.power_consumption {
            if power < 0.0 {
                violations.push("Negative power consumption violates energy conservation".to_string());
                energy_conserved = false;
            }
        }

        // Entropy constraint (entropy must decrease for XP generation)
        if input.entropy_reduction <= 0.0 {
            violations.push("Entropy must decrease for XP generation".to_string());
            entropy_constraint_satisfied = false;
        }

        // Causal ordering (closure speed must be positive and finite)
        if input.closure_speed <= 0.0 || !input.closure_speed.is_finite() {
            violations.push("Invalid causal closure speed".to_string());
            causal_ordering_preserved = false;
        }

        // Temperature constraint
        if let Some(temp) = input.temperature {
            if temp <= 0.0 {
                violations.push("Temperature must be positive (Kelvin)".to_string());
                energy_conserved = false;
            }
        }

        // Reputation bounds
        if input.reputation < 0.0 || input.reputation > 1.0 {
            violations.push("Reputation must be between 0 and 1".to_string());
        }

        let is_compliant = violations.is_empty();
        let compliance_confidence = if is_compliant { 1.0 } else { 0.0 };

        Ok(ThermodynamicCompliance {
            is_compliant,
            energy_conserved,
            entropy_constraint_satisfied,
            causal_ordering_preserved,
            violations,
            compliance_confidence,
        })
    }

    /// Calculate confidence in the XP calculation
    fn calculate_confidence(&self, input: &XpCalculationInput) -> f64 {
        let mut confidence = 1.0;

        // Reduce confidence for low reputation
        if input.reputation < 0.5 {
            confidence *= 0.8;
        }

        // Reduce confidence for weak feedback closure
        if input.feedback_closure < 0.7 {
            confidence *= 0.9;
        }

        // Reduce confidence for high temporal uncertainty
        if input.temporal_sustainability < 0.3 {
            confidence *= 0.85;
        }

        // Increase confidence for strong network effects
        if input.network_impact > 1.5 {
            confidence *= 1.1;
        }

        confidence.min(1.0).max(0.1)
    }

    /// Determine primary domain context
    fn determine_domain_context(&self, input: &XpCalculationInput) -> String {
        if input.domain_vector.is_empty() {
            return "general".to_string();
        }

        let max_index = input.domain_vector.iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(i, _)| i)
            .unwrap_or(0);

        match max_index {
            0 => "cognitive".to_string(),
            1 => "physical".to_string(),
            2 => "social".to_string(),
            3 => "economic".to_string(),
            4 => "creative".to_string(),
            5 => "system".to_string(),
            _ => "other".to_string(),
        }
    }

    /// Detect cross-domain synergies
    fn detect_cross_domain_synergies(&self, input: &XpCalculationInput) -> Vec<String> {
        let mut synergies = Vec::new();
        let threshold = 0.3;

        let domains = ["cognitive", "physical", "social", "economic", "creative", "system"];
        let active_domains: Vec<_> = input.domain_vector.iter()
            .enumerate()
            .filter(|(_, &weight)| weight > threshold)
            .map(|(i, _)| domains.get(i).unwrap_or(&"unknown"))
            .collect();

        if active_domains.len() > 1 {
            synergies.push(format!("multi_domain_{}", active_domains.len()));
            
            // Specific synergy patterns
            if active_domains.contains(&"cognitive") && active_domains.contains(&"creative") {
                synergies.push("cognitive_creative_synergy".to_string());
            }
            if active_domains.contains(&"social") && active_domains.contains(&"system") {
                synergies.push("social_system_synergy".to_string());
            }
            if active_domains.contains(&"physical") && active_domains.contains(&"economic") {
                synergies.push("physical_economic_synergy".to_string());
            }
        }

        synergies
    }

    /// Validate input parameters
    fn validate_input(&self, input: &XpCalculationInput) -> Result<()> {
        if input.reputation < 0.0 || input.reputation > 1.0 {
            return Err(Error::Other("Reputation must be between 0 and 1".into()));
        }
        if input.feedback_closure < 0.0 || input.feedback_closure > 1.0 {
            return Err(Error::Other("Feedback closure must be between 0 and 1".into()));
        }
        if input.entropy_reduction <= 0.0 {
            return Err(Error::Other("Entropy reduction must be positive".into()));
        }
        if input.closure_speed <= 0.0 {
            return Err(Error::Other("Closure speed must be positive".into()));
        }
        if input.temporal_sustainability <= 0.0 {
            return Err(Error::Other("Temporal sustainability must be positive".into()));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::temporal::XpTime;

    #[test]
    fn test_basic_xp_calculation() {
        let calculator = PhysicsXpCalculator::default();
        let input = XpCalculationInput {
            reputation: 0.8,
            feedback_closure: 0.9,
            entropy_reduction: 2.5,
            domain_vector: vec![0.8, 0.2, 0.1, 0.0, 0.0, 0.0],
            essentiality_vector: vec![0.9, 0.1, 0.1, 0.0, 0.0, 0.0],
            temporal_sustainability: 0.7,
            closure_speed: 1.2,
            power_consumption: Some(100.0),
            temperature: Some(298.15),
            network_impact: 1.0,
            collaboration_factor: 1.0,
            calculation_time: XpTime::new(2024, 100, 5, 50).unwrap(),
            validator: None,
        };

        let result = calculator.calculate_xp(&input).unwrap();
        assert!(result.xp_amount > 0);
        assert!(result.thermodynamic_compliance.is_compliant);
        assert_eq!(result.metadata.formula_type, XpFormulaType::Complete);
    }

    #[test]
    fn test_thermodynamic_compliance_violation() {
        let calculator = PhysicsXpCalculator::default();
        let input = XpCalculationInput {
            reputation: 0.8,
            feedback_closure: 0.9,
            entropy_reduction: -1.0, // Negative entropy reduction
            domain_vector: vec![1.0],
            essentiality_vector: vec![1.0],
            temporal_sustainability: 0.7,
            closure_speed: 1.2,
            power_consumption: None,
            temperature: None,
            network_impact: 1.0,
            collaboration_factor: 1.0,
            calculation_time: XpTime::new(2024, 100, 5, 50).unwrap(),
            validator: None,
        };

        assert!(calculator.calculate_xp(&input).is_err());
    }

    #[test]
    fn test_cross_domain_synergies() {
        let calculator = PhysicsXpCalculator::default();
        let input = XpCalculationInput {
            reputation: 0.8,
            feedback_closure: 0.9,
            entropy_reduction: 2.5,
            domain_vector: vec![0.8, 0.1, 0.7, 0.0, 0.9, 0.0], // Multiple active domains
            essentiality_vector: vec![0.9, 0.1, 0.8, 0.0, 0.8, 0.0],
            temporal_sustainability: 0.7,
            closure_speed: 1.2,
            power_consumption: Some(100.0),
            temperature: Some(298.15),
            network_impact: 1.0,
            collaboration_factor: 1.0,
            calculation_time: XpTime::new(2024, 100, 5, 50).unwrap(),
            validator: None,
        };

        let result = calculator.calculate_xp(&input).unwrap();
        assert!(!result.metadata.cross_domain_synergies.is_empty());
        assert!(result.metadata.cross_domain_synergies.contains(&"cognitive_creative_synergy".to_string()));
    }
}