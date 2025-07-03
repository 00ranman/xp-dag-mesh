use crate::messages::{P2PMessage, GossipTopic, PropagationStrategy};
use common::{Hash, Address};
use dag_core::Transaction;
use validator_mesh::{ValidationRequest, ValidationResponse};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, warn};

/// Gossip protocol manager
pub struct GossipManager {
    seen_messages: Arc<RwLock<LruCache<Hash, Instant>>>,
    topic_subscribers: Arc<RwLock<HashMap<GossipTopic, HashSet<Address>>>>,
    message_ttl: Duration,
    max_cache_size: usize,
}

impl GossipManager {
    pub fn new(message_ttl: Duration, max_cache_size: usize) -> Self {
        Self {
            seen_messages: Arc::new(RwLock::new(LruCache::new(max_cache_size))),
            topic_subscribers: Arc::new(RwLock::new(HashMap::new())),
            message_ttl,
            max_cache_size,
        }
    }
    
    /// Check if we've seen this message recently
    pub async fn is_duplicate(&self, msg_hash: &Hash) -> bool {
        let mut cache = self.seen_messages.write().await;
        
        if let Some(timestamp) = cache.get(msg_hash) {
            if timestamp.elapsed() < self.message_ttl {
                return true;
            }
        }
        
        // Mark as seen
        cache.insert(*msg_hash, Instant::now());
        false
    }
    
    /// Subscribe an address to a topic
    pub async fn subscribe(&self, topic: GossipTopic, address: Address) {
        let mut subscribers = self.topic_subscribers.write().await;
        subscribers.entry(topic).or_insert_with(HashSet::new).insert(address);
    }
    
    /// Unsubscribe an address from a topic
    pub async fn unsubscribe(&self, topic: GossipTopic, address: Address) {
        let mut subscribers = self.topic_subscribers.write().await;
        if let Some(addresses) = subscribers.get_mut(&topic) {
            addresses.remove(&address);
        }
    }
    
    /// Get subscribers for a topic
    pub async fn get_subscribers(&self, topic: &GossipTopic) -> Vec<Address> {
        let subscribers = self.topic_subscribers.read().await;
        subscribers.get(topic)
            .map(|s| s.iter().copied().collect())
            .unwrap_or_default()
    }
    
    /// Determine which topic a message belongs to
    pub fn message_topic(message: &P2PMessage) -> GossipTopic {
        match message {
            P2PMessage::NewTransaction { .. } => GossipTopic::Transactions,
            P2PMessage::RequestValidation { .. } |
            P2PMessage::ValidationResult { .. } => GossipTopic::Validations,
            P2PMessage::Tips { .. } => GossipTopic::Tips,
            P2PMessage::StateSnapshot { .. } => GossipTopic::StateUpdates,
            _ => GossipTopic::Transactions, // Default
        }
    }
    
    /// Apply propagation filters
    pub fn should_propagate_to_peer(
        &self,
        message: &P2PMessage,
        peer_info: &PeerInfo,
        strategy: &PropagationStrategy,
    ) -> bool {
        match strategy {
            PropagationStrategy::ValidatorsOnly => {
                // Only send to validators
                peer_info.is_validator
            }
            
            PropagationStrategy::EntropyWeighted { threshold } => {
                // Send to peers with high entropy contribution
                peer_info.entropy_score >= *threshold
            }
            
            _ => true, // Other strategies handled elsewhere
        }
    }
}

/// Simple LRU cache implementation
struct LruCache<K: std::hash::Hash + Eq, V> {
    map: HashMap<K, (V, usize)>,
    order: Vec<K>,
    capacity: usize,
}

impl<K: std::hash::Hash + Eq + Clone, V> LruCache<K, V> {
    fn new(capacity: usize) -> Self {
        Self {
            map: HashMap::with_capacity(capacity),
            order: Vec::with_capacity(capacity),
            capacity,
        }
    }
    
    fn get(&mut self, key: &K) -> Option<&V> {
        if let Some((value, _)) = self.map.get(key) {
            // Move to end (most recently used)
            self.order.retain(|k| k != key);
            self.order.push(key.clone());
            return Some(value);
        }
        None
    }
    
    fn insert(&mut self, key: K, value: V) {
        if self.map.len() >= self.capacity && !self.map.contains_key(&key) {
            // Evict least recently used
            if let Some(lru_key) = self.order.first().cloned() {
                self.order.remove(0);
                self.map.remove(&lru_key);
            }
        }
        
        self.map.insert(key.clone(), (value, self.order.len()));
        self.order.push(key);
    }
}

/// Information about a peer
#[derive(Clone, Debug)]
pub struct PeerInfo {
    pub address: Address,
    pub is_validator: bool,
    pub entropy_score: f64,
    pub reputation: f64,
    pub last_seen: Instant,
}

/// Message validation before propagation
pub async fn validate_gossip_message(message: &P2PMessage) -> bool {
    match message {
        P2PMessage::NewTransaction { tx } => {
            // Basic transaction validation
            tx.hash == tx.compute_hash()
        }
        
        P2PMessage::ValidationResult { response } => {
            // Check signature validity
            // In real implementation, would verify validator signature
            true
        }
        
        _ => true, // Other messages don't need validation
    }
}

/// Calculate message priority for propagation
pub fn message_priority(message: &P2PMessage) -> u8 {
    match message {
        // High priority for validation results
        P2PMessage::ValidationResult { .. } => 10,
        
        // Medium-high for new transactions with entropy reduction
        P2PMessage::NewTransaction { tx } if tx.is_entropy_reducing() => 8,
        
        // Medium for regular transactions
        P2PMessage::NewTransaction { .. } => 5,
        
        // Low for sync/status messages
        P2PMessage::GetSyncStatus | P2PMessage::SyncStatus { .. } => 2,
        
        // Default medium priority
        _ => 5,
    }
}