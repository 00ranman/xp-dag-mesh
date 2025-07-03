use common::{Hash, Address, XpAmount};
use dag_core::Transaction;
use validator_mesh::{ValidationRequest, ValidationResponse};
use serde::{Deserialize, Serialize};

/// P2P message types
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum P2PMessage {
    /// Request specific transactions
    GetTransactions { hashes: Vec<Hash> },
    
    /// Response with transactions
    Transactions { txs: Vec<Transaction> },
    
    /// Request DAG tips
    GetTips,
    
    /// Response with current tips
    Tips { tips: Vec<Hash> },
    
    /// New transaction announcement
    NewTransaction { tx: Transaction },
    
    /// Request validation
    RequestValidation { request: ValidationRequest },
    
    /// Validation response
    ValidationResult { response: ValidationResponse },
    
    /// Sync status request
    GetSyncStatus,
    
    /// Sync status response
    SyncStatus { status: NodeSyncStatus },
    
    /// Request state snapshot
    GetStateSnapshot { block_height: u64 },
    
    /// State snapshot response
    StateSnapshot { snapshot: StateSnapshotData },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeSyncStatus {
    pub latest_tips: Vec<Hash>,
    pub dag_size: u64,
    pub total_xp_minted: XpAmount,
    pub active_validators: u32,
    pub sync_progress: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StateSnapshotData {
    pub block_height: u64,
    pub state_root: Hash,
    pub account_count: u64,
    pub contract_count: u64,
    pub total_entropy_reduced: f64,
}

/// Transaction propagation strategy
#[derive(Clone, Debug, PartialEq)]
pub enum PropagationStrategy {
    /// Flood to all peers
    Flood,
    /// Send to sqrt(n) random peers
    RandomWalk { factor: f64 },
    /// Send only to validators
    ValidatorsOnly,
    /// Entropy-weighted propagation
    EntropyWeighted { threshold: f64 },
}

/// Sync request for catching up
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SyncRequest {
    /// Starting point (exclusive)
    pub from_tips: Vec<Hash>,
    /// Maximum number of transactions to return
    pub max_txs: u32,
    /// Include orphaned branches
    pub include_orphans: bool,
}

/// Sync response with DAG fragment
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SyncResponse {
    /// Transactions in topological order
    pub transactions: Vec<Transaction>,
    /// Current tips after these transactions
    pub new_tips: Vec<Hash>,
    /// Whether more data is available
    pub has_more: bool,
}

/// Gossip topic for different message types
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum GossipTopic {
    Transactions,
    Validations,
    Tips,
    StateUpdates,
}

impl GossipTopic {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Transactions => "/xpnet/txs/1.0.0",
            Self::Validations => "/xpnet/vals/1.0.0",
            Self::Tips => "/xpnet/tips/1.0.0",
            Self::StateUpdates => "/xpnet/state/1.0.0",
        }
    }
}