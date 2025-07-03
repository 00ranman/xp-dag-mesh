use serde::{Deserialize, Serialize};
use crate::types::{Hash, Address, Result, Error};
use crate::temporal::{XpTime, TemporalLoop};
use crate::entropy::EntropyReduction;

/// Retroactive XP minting system for closed causal loops

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CausalLoop {
    /// Unique loop identifier
    pub id: Hash,
    /// Loop initiator
    pub initiator: Address,
    /// Type of causal loop
    pub loop_type: CausalLoopType,
    /// Start time of the loop
    pub start_time: XpTime,
    /// End time (when loop closed)
    pub end_time: Option<XpTime>,
    /// Platform where loop originated
    pub platform: Platform,
    /// Initial entropy state
    pub initial_entropy: f64,
    /// Final entropy state (when closed)
    pub final_entropy: Option<f64>,
    /// Calculated entropy reduction
    pub entropy_reduction: Option<f64>,
    /// Current loop state
    pub loop_state: LoopState,
    /// Chain of causal events
    pub causal_chain: Vec<CausalEvent>,
    /// Retroactive XP amount calculated
    pub retroactive_xp: f64,
    /// Bonus multiplier for retroactive minting
    pub bonus_multiplier: f64,
    /// Validation score from consensus
    pub validation_score: Option<f64>,
    /// When closure was detected
    pub closure_detected: Option<XpTime>,
    /// When retroactive XP was minted
    pub retroactive_minted: Option<XpTime>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum CausalLoopType {
    /// Cross-platform integration loop
    CrossPlatform,
    /// LevelUp Academy skill development loop
    LevelUp,
    /// SignalFlow task orchestration loop
    SignalFlow,
    /// XP Timekeeping temporal loop
    Timekeeping,
    /// Merchant network value loop
    Merchant,
    /// Validator consensus loop
    Validator,
    /// Custom loop type
    Custom { name: String },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Platform {
    /// LevelUp Academy
    Academy,
    /// SignalFlow task orchestration
    SignalFlow,
    /// XP Timekeeping system
    Timekeeping,
    /// Merchant network
    Merchant,
    /// Unified cross-platform
    Unified,
    /// External platform
    External { name: String },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum LoopState {
    /// Loop is open and active
    Open,
    /// Loop is in the process of closing
    Closing,
    /// Loop has been closed
    Closed,
    /// Loop closure has been validated
    Validated,
    /// Loop was abandoned or failed
    Failed { reason: String },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CausalEvent {
    /// Event identifier
    pub id: Hash,
    /// Type of causal event
    pub event_type: CausalEventType,
    /// Platform where event occurred
    pub platform: Platform,
    /// Event-specific data
    pub event_data: serde_json::Value,
    /// Entropy change from this event
    pub entropy_delta: f64,
    /// Timestamp of the event
    pub timestamp: XpTime,
    /// Whether event has been processed
    pub processed: bool,
    /// Retroactive XP attributed to this event
    pub retroactive_xp: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum CausalEventType {
    /// Contribution made
    Contribution,
    /// Task completed
    TaskCompletion,
    /// Skill emergence detected
    SkillEmergence,
    /// Temporal closure achieved
    TemporalClosure,
    /// Collaboration initiated
    CollaborationStart,
    /// Validation consensus reached
    ValidationConsensus,
    /// XP minting event
    XpMinting,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RetroactiveXpTransaction {
    /// Transaction identifier
    pub id: Hash,
    /// User receiving retroactive XP
    pub user_id: Address,
    /// Original XP amount awarded
    pub original_xp: f64,
    /// Additional retroactive XP
    pub retroactive_xp: f64,
    /// Reason for the retroactive bonus
    pub bonus_reason: String,
    /// Associated causal loop
    pub loop_id: Option<Hash>,
    /// Platform that triggered the bonus
    pub platform_origin: Platform,
    /// Multiplier applied
    pub multiplier: f64,
    /// Validation status
    pub validation_status: ValidationStatus,
    /// When retroactive XP was minted
    pub minted_at: XpTime,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ValidationStatus {
    /// Pending validation
    Pending,
    /// Validated and approved
    Validated,
    /// Rejected or burned
    Burned,
    /// Under review
    UnderReview,
}

/// Retroactive XP calculation engine
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RetroactiveXpEngine {
    /// Base multiplier for retroactive bonuses
    pub base_multiplier: f64,
    /// Cross-platform bonus factor
    pub cross_platform_bonus: f64,
    /// Time decay factor for older loops
    pub time_decay_factor: f64,
    /// Minimum loop duration for retroactive minting
    pub min_loop_duration_minutes: u64,
    /// Maximum retroactive bonus percentage
    pub max_bonus_percentage: f64,
}

impl Default for RetroactiveXpEngine {
    fn default() -> Self {
        Self {
            base_multiplier: 1.15,
            cross_platform_bonus: 1.25,
            time_decay_factor: 0.95,
            min_loop_duration_minutes: 60, // 1 hour minimum
            max_bonus_percentage: 0.5, // 50% max bonus
        }
    }
}

impl RetroactiveXpEngine {
    /// Calculate retroactive XP for a closed causal loop
    pub fn calculate_retroactive_xp(&self, causal_loop: &CausalLoop) -> Result<f64> {
        // Validate loop can generate retroactive XP
        if causal_loop.loop_state != LoopState::Closed {
            return Err(Error::Other("Loop must be closed to calculate retroactive XP".into()));
        }

        let entropy_reduction = causal_loop.entropy_reduction
            .ok_or_else(|| Error::Other("No entropy reduction calculated".into()))?;

        if entropy_reduction <= 0.0 {
            return Err(Error::Other("Entropy reduction must be positive".into()));
        }

        // Calculate base retroactive XP from entropy reduction
        let base_xp = entropy_reduction * 100.0; // Base conversion factor

        // Apply base multiplier
        let mut retroactive_xp = base_xp * self.base_multiplier;

        // Cross-platform bonus
        if causal_loop.loop_type == CausalLoopType::CrossPlatform {
            retroactive_xp *= self.cross_platform_bonus;
        }

        // Time decay factor (longer loops get less bonus)
        if let (Some(start), Some(end)) = (Some(causal_loop.start_time), causal_loop.end_time) {
            let duration_minutes = end.to_total_minutes() - start.to_total_minutes();
            
            if duration_minutes < self.min_loop_duration_minutes {
                return Ok(0.0); // Too short for bonus
            }

            // Apply exponential decay for very long loops
            let decay_factor = self.time_decay_factor.powf((duration_minutes as f64) / 1440.0); // Per day
            retroactive_xp *= decay_factor;
        }

        // Validation score bonus
        if let Some(validation_score) = causal_loop.validation_score {
            retroactive_xp *= validation_score;
        }

        // Cap the bonus
        let original_xp: f64 = causal_loop.causal_chain.iter()
            .map(|event| event.retroactive_xp)
            .sum();
        
        let max_bonus = original_xp * self.max_bonus_percentage;
        retroactive_xp = retroactive_xp.min(max_bonus);

        Ok(retroactive_xp)
    }

    /// Detect when a causal loop should be closed
    pub fn detect_loop_closure(
        &self,
        causal_loop: &mut CausalLoop,
        new_event: CausalEvent,
    ) -> Result<bool> {
        // Add the new event to the causal chain
        causal_loop.causal_chain.push(new_event);

        // Check for closure conditions based on loop type
        match causal_loop.loop_type {
            CausalLoopType::CrossPlatform => {
                self.detect_cross_platform_closure(causal_loop)
            }
            CausalLoopType::LevelUp => {
                self.detect_levelup_closure(causal_loop)
            }
            CausalLoopType::SignalFlow => {
                self.detect_signalflow_closure(causal_loop)
            }
            CausalLoopType::Timekeeping => {
                self.detect_timekeeping_closure(causal_loop)
            }
            _ => Ok(false), // Custom loops need manual closure
        }
    }

    /// Detect cross-platform loop closure
    fn detect_cross_platform_closure(&self, causal_loop: &mut CausalLoop) -> Result<bool> {
        // Check if we have events from multiple platforms
        let platforms: std::collections::HashSet<_> = causal_loop.causal_chain
            .iter()
            .map(|event| &event.platform)
            .collect();

        if platforms.len() >= 2 {
            // Check for skill emergence or contribution validation across platforms
            let has_skill_emergence = causal_loop.causal_chain.iter()
                .any(|event| event.event_type == CausalEventType::SkillEmergence);
            
            let has_validation = causal_loop.causal_chain.iter()
                .any(|event| event.event_type == CausalEventType::ValidationConsensus);

            if has_skill_emergence && has_validation {
                self.close_loop(causal_loop)?;
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Detect LevelUp Academy loop closure
    fn detect_levelup_closure(&self, causal_loop: &mut CausalLoop) -> Result<bool> {
        // Look for skill mastery completion
        let skill_events: Vec<_> = causal_loop.causal_chain.iter()
            .filter(|event| event.event_type == CausalEventType::SkillEmergence)
            .collect();

        if skill_events.len() >= 1 {
            // Check if skill has been validated by other users or platforms
            let validated = causal_loop.causal_chain.iter()
                .any(|event| event.event_type == CausalEventType::ValidationConsensus);

            if validated {
                self.close_loop(causal_loop)?;
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Detect SignalFlow loop closure
    fn detect_signalflow_closure(&self, causal_loop: &mut CausalLoop) -> Result<bool> {
        // Look for task completion followed by XP minting
        let mut has_task_completion = false;
        let mut has_xp_minting = false;

        for event in &causal_loop.causal_chain {
            match event.event_type {
                CausalEventType::TaskCompletion => has_task_completion = true,
                CausalEventType::XpMinting => has_xp_minting = true,
                _ => {}
            }
        }

        if has_task_completion && has_xp_minting {
            self.close_loop(causal_loop)?;
            return Ok(true);
        }

        Ok(false)
    }

    /// Detect timekeeping loop closure
    fn detect_timekeeping_closure(&self, causal_loop: &mut CausalLoop) -> Result<bool> {
        // Look for temporal closure events
        let has_temporal_closure = causal_loop.causal_chain.iter()
            .any(|event| event.event_type == CausalEventType::TemporalClosure);

        if has_temporal_closure {
            self.close_loop(causal_loop)?;
            return Ok(true);
        }

        Ok(false)
    }

    /// Close a causal loop and calculate final entropy
    fn close_loop(&self, causal_loop: &mut CausalLoop) -> Result<()> {
        causal_loop.loop_state = LoopState::Closed;
        causal_loop.end_time = Some(XpTime::now());
        causal_loop.closure_detected = Some(XpTime::now());

        // Calculate total entropy reduction from all events
        let total_entropy_delta: f64 = causal_loop.causal_chain.iter()
            .map(|event| event.entropy_delta)
            .sum();

        causal_loop.final_entropy = Some(causal_loop.initial_entropy + total_entropy_delta);
        causal_loop.entropy_reduction = Some(-total_entropy_delta); // Negative delta = entropy reduction

        // Calculate retroactive XP
        causal_loop.retroactive_xp = self.calculate_retroactive_xp(causal_loop)?;

        Ok(())
    }

    /// Create retroactive XP transaction
    pub fn create_retroactive_transaction(
        &self,
        causal_loop: &CausalLoop,
        original_xp: f64,
    ) -> Result<RetroactiveXpTransaction> {
        if causal_loop.loop_state != LoopState::Closed {
            return Err(Error::Other("Loop must be closed to create retroactive transaction".into()));
        }

        Ok(RetroactiveXpTransaction {
            id: Hash::new(rand::random()),
            user_id: causal_loop.initiator,
            original_xp,
            retroactive_xp: causal_loop.retroactive_xp,
            bonus_reason: format!("{:?} causal loop closure", causal_loop.loop_type),
            loop_id: Some(causal_loop.id),
            platform_origin: causal_loop.platform.clone(),
            multiplier: causal_loop.bonus_multiplier,
            validation_status: ValidationStatus::Pending,
            minted_at: XpTime::now(),
        })
    }
}

impl CausalLoop {
    /// Create a new causal loop
    pub fn new(
        initiator: Address,
        loop_type: CausalLoopType,
        platform: Platform,
        initial_entropy: f64,
    ) -> Self {
        Self {
            id: Hash::new(rand::random()),
            initiator,
            loop_type,
            start_time: XpTime::now(),
            end_time: None,
            platform,
            initial_entropy,
            final_entropy: None,
            entropy_reduction: None,
            loop_state: LoopState::Open,
            causal_chain: Vec::new(),
            retroactive_xp: 0.0,
            bonus_multiplier: 1.0,
            validation_score: None,
            closure_detected: None,
            retroactive_minted: None,
        }
    }

    /// Add an event to the causal chain
    pub fn add_event(&mut self, event: CausalEvent) {
        self.causal_chain.push(event);
    }

    /// Get total entropy delta from all events
    pub fn total_entropy_delta(&self) -> f64 {
        self.causal_chain.iter().map(|e| e.entropy_delta).sum()
    }

    /// Check if loop has been closed
    pub fn is_closed(&self) -> bool {
        matches!(self.loop_state, LoopState::Closed | LoopState::Validated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_causal_loop_creation() {
        let loop_ = CausalLoop::new(
            Address::new([1; 20]),
            CausalLoopType::CrossPlatform,
            Platform::Unified,
            5.0,
        );

        assert_eq!(loop_.loop_state, LoopState::Open);
        assert_eq!(loop_.initial_entropy, 5.0);
        assert!(loop_.causal_chain.is_empty());
    }

    #[test]
    fn test_retroactive_xp_calculation() {
        let engine = RetroactiveXpEngine::default();
        let mut loop_ = CausalLoop::new(
            Address::new([1; 20]),
            CausalLoopType::CrossPlatform,
            Platform::Unified,
            5.0,
        );

        // Close the loop manually for testing
        loop_.loop_state = LoopState::Closed;
        loop_.entropy_reduction = Some(2.0);
        loop_.validation_score = Some(0.9);

        let retroactive_xp = engine.calculate_retroactive_xp(&loop_).unwrap();
        assert!(retroactive_xp > 0.0);
    }

    #[test]
    fn test_retroactive_transaction_creation() {
        let engine = RetroactiveXpEngine::default();
        let mut loop_ = CausalLoop::new(
            Address::new([1; 20]),
            CausalLoopType::LevelUp,
            Platform::Academy,
            5.0,
        );

        // Close the loop
        loop_.loop_state = LoopState::Closed;
        loop_.entropy_reduction = Some(1.5);
        loop_.retroactive_xp = 50.0;

        let transaction = engine.create_retroactive_transaction(&loop_, 100.0).unwrap();
        assert_eq!(transaction.user_id, loop_.initiator);
        assert_eq!(transaction.original_xp, 100.0);
        assert_eq!(transaction.retroactive_xp, 50.0);
    }
}