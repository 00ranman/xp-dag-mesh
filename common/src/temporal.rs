use serde::{Deserialize, Serialize};
use std::fmt;
use crate::types::{Result, Error, Hash};

/// Base-10 temporal architecture for XP timekeeping system
/// Implements 10-hour days, 5-day weeks, 15-day months, 24 months per year

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct XpTime {
    /// Year in XP calendar
    pub year: u32,
    /// Day of year (0-359, 360 days per year)
    pub day: u16,
    /// Hour of day (0-9, 10 hours per day)
    pub hour: u8,
    /// Minute of hour (0-99, 100 minutes per hour)
    pub minute: u8,
}

impl XpTime {
    /// Create new XP time
    pub fn new(year: u32, day: u16, hour: u8, minute: u8) -> Result<Self> {
        if day >= 360 {
            return Err(Error::Other("Day must be 0-359".into()));
        }
        if hour >= 10 {
            return Err(Error::Other("Hour must be 0-9".into()));
        }
        if minute >= 100 {
            return Err(Error::Other("Minute must be 0-99".into()));
        }
        
        Ok(Self { year, day, hour, minute })
    }

    /// Current XP time
    pub fn now() -> Self {
        let gregorian_epoch = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        Self::from_gregorian_timestamp(gregorian_epoch)
    }

    /// Convert from Gregorian timestamp (Unix epoch)
    pub fn from_gregorian_timestamp(timestamp: u64) -> Self {
        // XP epoch starts at 2024-01-01 00:00:00 UTC
        const XP_EPOCH: u64 = 1704067200; // 2024-01-01 in Unix timestamp
        
        let xp_seconds = timestamp.saturating_sub(XP_EPOCH);
        
        // Base-10 time conversion
        // 1 XP day = 26.4 hours = 95,040 seconds (includes drift compensation)
        const XP_DAY_SECONDS: u64 = 95_040;
        const XP_HOUR_SECONDS: u64 = 9_504; // 100 XP minutes
        const XP_MINUTE_SECONDS: u64 = 95;  // Slightly longer than Gregorian minute
        
        let total_xp_days = xp_seconds / XP_DAY_SECONDS;
        let year = 2024 + (total_xp_days / 360) as u32;
        let day = (total_xp_days % 360) as u16;
        
        let day_remainder = xp_seconds % XP_DAY_SECONDS;
        let hour = (day_remainder / XP_HOUR_SECONDS) as u8;
        let hour_remainder = day_remainder % XP_HOUR_SECONDS;
        let minute = (hour_remainder / XP_MINUTE_SECONDS) as u8;
        
        Self { year, day, hour, minute }
    }

    /// Convert to Gregorian timestamp
    pub fn to_gregorian_timestamp(&self) -> u64 {
        const XP_EPOCH: u64 = 1704067200; // 2024-01-01 in Unix timestamp
        const XP_DAY_SECONDS: u64 = 95_040;
        const XP_HOUR_SECONDS: u64 = 9_504;
        const XP_MINUTE_SECONDS: u64 = 95;
        
        let years_since_epoch = self.year - 2024;
        let total_days = (years_since_epoch as u64) * 360 + self.day as u64;
        let total_seconds = total_days * XP_DAY_SECONDS 
            + (self.hour as u64) * XP_HOUR_SECONDS 
            + (self.minute as u64) * XP_MINUTE_SECONDS;
        
        XP_EPOCH + total_seconds
    }

    /// Get the week number (0-71, 72 weeks per year)
    pub fn week(&self) -> u8 {
        (self.day / 5) as u8
    }

    /// Get the day of week (0-4, 5 days per week)
    pub fn day_of_week(&self) -> u8 {
        (self.day % 5) as u8
    }

    /// Get the month number (0-23, 24 months per year)
    pub fn month(&self) -> u8 {
        (self.day / 15) as u8
    }

    /// Get the day of month (0-14, 15 days per month)
    pub fn day_of_month(&self) -> u8 {
        (self.day % 15) as u8
    }

    /// Add duration in XP time units
    pub fn add_minutes(&self, minutes: u64) -> Self {
        let total_minutes = self.to_total_minutes() + minutes;
        Self::from_total_minutes(total_minutes)
    }

    /// Convert to total minutes since XP epoch
    pub fn to_total_minutes(&self) -> u64 {
        let years_since_epoch = self.year - 2024;
        let total_days = (years_since_epoch as u64) * 360 + self.day as u64;
        total_days * 1000 + (self.hour as u64) * 100 + self.minute as u64
    }

    /// Create from total minutes since XP epoch
    pub fn from_total_minutes(total_minutes: u64) -> Self {
        let total_days = total_minutes / 1000;
        let year = 2024 + (total_days / 360) as u32;
        let day = (total_days % 360) as u16;
        
        let day_minutes = total_minutes % 1000;
        let hour = (day_minutes / 100) as u8;
        let minute = (day_minutes % 100) as u8;
        
        Self { year, day, hour, minute }
    }
}

impl fmt::Display for XpTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "XP:{}:{}:{:02}:{:02}", self.year, self.day, self.hour, self.minute)
    }
}

impl std::str::FromStr for XpTime {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 4 || parts[0] != "XP" {
            return Err(Error::Other("Invalid XP time format. Expected XP:YYYY:DDD:HH:MM".into()));
        }

        let year = parts[1].parse::<u32>()
            .map_err(|_| Error::Other("Invalid year".into()))?;
        let day = parts[2].parse::<u16>()
            .map_err(|_| Error::Other("Invalid day".into()))?;
        let hour = parts[3].parse::<u8>()
            .map_err(|_| Error::Other("Invalid hour".into()))?;
        let minute = parts[4].parse::<u8>()
            .map_err(|_| Error::Other("Invalid minute".into()))?;

        Self::new(year, day, hour, minute)
    }
}

/// DAG-based temporal structure for nested time relationships
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TemporalDag {
    /// Unique identifier for this temporal structure
    pub id: Hash,
    /// Type of temporal structure
    pub structure_type: TemporalStructureType,
    /// Nodes in the temporal DAG
    pub nodes: Vec<TemporalNode>,
    /// Edges representing causal relationships
    pub edges: Vec<TemporalEdge>,
    /// Root node for traversal
    pub root_node: Hash,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum TemporalStructureType {
    /// Linear sequence of events
    Sequential,
    /// Parallel branches that can merge
    Branching,
    /// Cyclical pattern with feedback
    Cyclical,
    /// Complex nested structure
    Fractal { depth: u8 },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TemporalNode {
    /// Node identifier
    pub id: Hash,
    /// XP time of this node
    pub xp_time: XpTime,
    /// Type of temporal event
    pub event_type: TemporalEventType,
    /// Associated data
    pub data: serde_json::Value,
    /// Parent nodes (can have multiple for DAG structure)
    pub parents: Vec<Hash>,
    /// Entropy state at this time
    pub entropy_state: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum TemporalEventType {
    /// Loop initiation
    LoopStart { loop_type: String },
    /// Loop closure
    LoopEnd { loop_type: String, entropy_reduction: f64 },
    /// Milestone or checkpoint
    Milestone { name: String },
    /// Causal dependency
    Dependency { depends_on: Vec<Hash> },
    /// Emergence event
    Emergence { complexity_increase: f64 },
    /// Retroactive correction
    Retroactive { original_time: XpTime, correction: String },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TemporalEdge {
    /// Source node
    pub from: Hash,
    /// Target node
    pub to: Hash,
    /// Type of causal relationship
    pub relation_type: CausalRelationType,
    /// Strength of causal connection (0.0 to 1.0)
    pub strength: f64,
    /// Time delay between cause and effect
    pub delay_minutes: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum CausalRelationType {
    /// Direct causal sequence
    Sequence,
    /// Enabling condition
    Enablement,
    /// Feedback loop
    Feedback,
    /// Parallel execution
    Parallel,
    /// Retroactive causation
    Retroactive,
}

/// Temporal loop for tracking causal closure
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TemporalLoop {
    /// Loop identifier
    pub id: Hash,
    /// Type of loop
    pub loop_type: LoopType,
    /// Start time in XP format
    pub start_time: XpTime,
    /// End time (None if loop is still open)
    pub end_time: Option<XpTime>,
    /// Planned duration in XP minutes
    pub planned_duration: u64,
    /// Initial entropy measurement
    pub initial_entropy: f64,
    /// Final entropy measurement (None if not closed)
    pub final_entropy: Option<f64>,
    /// Activities within the loop
    pub activities: Vec<TemporalActivity>,
    /// Current loop status
    pub status: LoopStatus,
    /// Associated temporal DAG
    pub temporal_dag: Option<TemporalDag>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum LoopType {
    /// Daily routine loop
    Daily,
    /// Weekly planning loop
    Weekly,
    /// Monthly review loop
    Monthly,
    /// Project-specific loop
    Project { project_id: String },
    /// Skill development loop
    Skill { skill_name: String },
    /// Custom loop
    Custom { name: String },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum LoopStatus {
    /// Loop is active and ongoing
    Active,
    /// Loop completed successfully
    Closed { entropy_reduction: f64 },
    /// Loop was abandoned or failed
    Abandoned { reason: String },
    /// Loop is paused temporarily
    Paused { pause_reason: String },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TemporalActivity {
    /// Activity identifier
    pub id: Hash,
    /// Activity name
    pub name: String,
    /// XP time when activity started
    pub start_time: XpTime,
    /// Duration in XP minutes
    pub duration: u64,
    /// Entropy change from this activity
    pub entropy_delta: f64,
    /// Activity metadata
    pub metadata: serde_json::Value,
}

impl TemporalLoop {
    /// Create a new temporal loop
    pub fn new(loop_type: LoopType, start_time: XpTime, initial_entropy: f64) -> Self {
        Self {
            id: Hash::new(rand::random()),
            loop_type,
            start_time,
            end_time: None,
            planned_duration: 0,
            initial_entropy,
            final_entropy: None,
            activities: Vec::new(),
            status: LoopStatus::Active,
            temporal_dag: None,
        }
    }

    /// Close the loop with final measurements
    pub fn close(&mut self, end_time: XpTime, final_entropy: f64) -> Result<f64> {
        if self.status != LoopStatus::Active {
            return Err(Error::Other("Loop is not active".into()));
        }

        let entropy_reduction = self.initial_entropy - final_entropy;
        self.end_time = Some(end_time);
        self.final_entropy = Some(final_entropy);
        self.status = LoopStatus::Closed { entropy_reduction };

        Ok(entropy_reduction)
    }

    /// Add activity to the loop
    pub fn add_activity(&mut self, activity: TemporalActivity) {
        self.activities.push(activity);
    }

    /// Calculate total entropy change from all activities
    pub fn total_entropy_delta(&self) -> f64 {
        self.activities.iter().map(|a| a.entropy_delta).sum()
    }

    /// Get loop duration in XP minutes
    pub fn duration_minutes(&self) -> Option<u64> {
        self.end_time.map(|end| {
            end.to_total_minutes() - self.start_time.to_total_minutes()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xp_time_creation() {
        let xp_time = XpTime::new(2024, 100, 5, 50).unwrap();
        assert_eq!(xp_time.year, 2024);
        assert_eq!(xp_time.day, 100);
        assert_eq!(xp_time.hour, 5);
        assert_eq!(xp_time.minute, 50);
    }

    #[test]
    fn test_xp_time_formatting() {
        let xp_time = XpTime::new(2024, 156, 14, 30).unwrap();
        assert_eq!(xp_time.to_string(), "XP:2024:156:14:30");
    }

    #[test]
    fn test_xp_time_parsing() {
        let xp_time: XpTime = "XP:2024:156:14:30".parse().unwrap();
        assert_eq!(xp_time.year, 2024);
        assert_eq!(xp_time.day, 156);
        assert_eq!(xp_time.hour, 14);
        assert_eq!(xp_time.minute, 30);
    }

    #[test]
    fn test_temporal_calculations() {
        let xp_time = XpTime::new(2024, 100, 5, 50).unwrap();
        assert_eq!(xp_time.week(), 20); // day 100 / 5
        assert_eq!(xp_time.day_of_week(), 0); // day 100 % 5
        assert_eq!(xp_time.month(), 6); // day 100 / 15
        assert_eq!(xp_time.day_of_month(), 10); // day 100 % 15
    }

    #[test]
    fn test_temporal_loop() {
        let start_time = XpTime::new(2024, 100, 0, 0).unwrap();
        let mut loop_ = TemporalLoop::new(LoopType::Daily, start_time, 5.0);
        
        assert_eq!(loop_.status, LoopStatus::Active);
        
        let end_time = XpTime::new(2024, 100, 9, 99).unwrap();
        let entropy_reduction = loop_.close(end_time, 3.0).unwrap();
        
        assert_eq!(entropy_reduction, 2.0);
        assert!(matches!(loop_.status, LoopStatus::Closed { .. }));
    }
}