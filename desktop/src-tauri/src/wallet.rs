//! Wallet & identity management for mesh nodes
//! Implements quantitative finance models for tiered access

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Tiered access levels using quantitative finance scoring
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AccessTier {
    /// Observer: read-only mesh participation
    Observer,
    /// Contributor: can propose DAG vertices
    Contributor,
    /// Validator: can validate and sign blocks
    Validator,
    /// Coordinator: full network coordination rights
    Coordinator,
}

/// Risk-adjusted position for tier calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantPosition {
    pub stake_amount: f64,
    pub uptime_ratio: f64,
    pub validation_accuracy: f64,
    pub network_contribution: f64,
    pub time_weighted_score: f64,
}

impl QuantPosition {
    /// Kelly criterion for optimal resource allocation
    pub fn kelly_fraction(&self) -> f64 {
        let win_prob = self.validation_accuracy;
        let win_loss_ratio = self.network_contribution / (1.0 - self.uptime_ratio).max(0.01);
        win_prob - ((1.0 - win_prob) / win_loss_ratio)
    }

    /// Sharpe-like ratio for node performance
    pub fn performance_ratio(&self, risk_free_baseline: f64) -> f64 {
        let excess_return = self.time_weighted_score - risk_free_baseline;
        let volatility = (1.0 - self.uptime_ratio).max(0.001);
        excess_return / volatility
    }

    /// Mean-reversion score: how far from network average
    pub fn mean_reversion_signal(&self, network_avg: f64) -> f64 {
        (network_avg - self.time_weighted_score) / network_avg.max(0.001)
    }

    /// Markowitz-inspired diversification score
    pub fn diversification_score(&self, correlation_matrix: &[f64]) -> f64 {
        if correlation_matrix.is_empty() {
            return 1.0;
        }
        let avg_corr: f64 = correlation_matrix.iter().sum::<f64>() / correlation_matrix.len() as f64;
        1.0 - avg_corr.abs()
    }
}

/// Wallet identity and access management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wallet {
    pub node_id: String,
    pub public_key: Vec<u8>,
    pub tier: AccessTier,
    pub position: QuantPosition,
    pub metadata: HashMap<String, String>,
}

impl Wallet {
    pub fn new(node_id: String, public_key: Vec<u8>) -> Self {
        Self {
            node_id,
            public_key,
            tier: AccessTier::Observer,
            position: QuantPosition {
                stake_amount: 0.0,
                uptime_ratio: 0.0,
                validation_accuracy: 0.0,
                network_contribution: 0.0,
                time_weighted_score: 0.0,
            },
            metadata: HashMap::new(),
        }
    }

    /// Recalculate tier based on quantitative position metrics
    pub fn recalculate_tier(&mut self) {
        let kelly = self.position.kelly_fraction();
        let perf = self.position.performance_ratio(0.1);
        let composite = (kelly * 0.3) + (perf * 0.3)
            + (self.position.uptime_ratio * 0.2)
            + (self.position.validation_accuracy * 0.2);

        self.tier = match composite {
            x if x >= 0.85 => AccessTier::Coordinator,
            x if x >= 0.60 => AccessTier::Validator,
            x if x >= 0.30 => AccessTier::Contributor,
            _ => AccessTier::Observer,
        };
    }

    /// Check if wallet has sufficient tier for an operation
    pub fn can_perform(&self, required: &AccessTier) -> bool {
        let self_level = match self.tier {
            AccessTier::Observer => 0,
            AccessTier::Contributor => 1,
            AccessTier::Validator => 2,
            AccessTier::Coordinator => 3,
        };
        let required_level = match required {
            AccessTier::Observer => 0,
            AccessTier::Contributor => 1,
            AccessTier::Validator => 2,
            AccessTier::Coordinator => 3,
        };
        self_level >= required_level
    }
}
