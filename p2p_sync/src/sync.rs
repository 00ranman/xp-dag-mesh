use crate::messages::{P2PMessage, SyncRequest, SyncResponse, PropagationStrategy};
use common::{Hash, Result, Error};
use dag_core::{Dag, Transaction};
use std::collections::{HashSet, VecDeque};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{info, debug, warn};

/// DAG synchronization manager
pub struct SyncManager {
    dag: Arc<Dag>,
    pending_txs: Arc<RwLock<HashSet<Hash>>>,
    sync_queue: Arc<RwLock<VecDeque<SyncRequest>>>,
    propagation_strategy: PropagationStrategy,
}

impl SyncManager {
    pub fn new(dag: Arc<Dag>, propagation_strategy: PropagationStrategy) -> Self {
        Self {
            dag,
            pending_txs: Arc::new(RwLock::new(HashSet::new())),
            sync_queue: Arc::new(RwLock::new(VecDeque::new())),
            propagation_strategy,
        }
    }
    
    /// Process incoming transaction
    pub async fn process_transaction(&self, tx: Transaction) -> Result<bool> {
        let hash = tx.hash;
        
        // Check if we already have it
        if self.dag.contains_transaction(&hash) {
            return Ok(false);
        }
        
        // Check if all parents exist
        let missing_parents = self.find_missing_parents(&tx).await;
        
        if missing_parents.is_empty() {
            // All parents exist, add to DAG
            self.dag.add_transaction(tx)?;
            
            // Remove from pending set
            self.pending_txs.write().await.remove(&hash);
            
            info!("Added transaction {} to DAG", hash);
            Ok(true)
        } else {
            // Missing parents, add to pending set
            self.pending_txs.write().await.insert(hash);
            
            debug!("Transaction {} pending, missing {} parents", hash, missing_parents.len());
            
            // Request missing parents
            self.request_transactions(missing_parents).await;
            
            Ok(false)
        }
    }
    
    /// Find missing parent transactions
    async fn find_missing_parents(&self, tx: &Transaction) -> Vec<Hash> {
        let mut missing = Vec::new();
        
        for parent_hash in &tx.parents {
            if !self.dag.contains_transaction(parent_hash) {
                missing.push(*parent_hash);
            }
        }
        
        missing
    }
    
    /// Request specific transactions
    async fn request_transactions(&self, hashes: Vec<Hash>) {
        // In real implementation, would send request to peers
        debug!("Requesting {} transactions", hashes.len());
    }
    
    /// Handle sync request from peer
    pub async fn handle_sync_request(&self, request: SyncRequest) -> Result<SyncResponse> {
        let mut transactions = Vec::new();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        
        // Start from current tips if no starting point specified
        let start_points = if request.from_tips.is_empty() {
            self.dag.get_tips()
        } else {
            request.from_tips
        };
        
        // BFS traversal
        for tip in start_points {
            queue.push_back(tip);
        }
        
        while let Some(hash) = queue.pop_front() {
            if visited.contains(&hash) {
                continue;
            }
            
            if let Some(tx) = self.dag.get_transaction(&hash) {
                visited.insert(hash);
                transactions.push((*tx).clone());
                
                // Add parents to queue
                for parent in &tx.parents {
                    if !visited.contains(parent) {
                        queue.push_back(*parent);
                    }
                }
                
                // Stop if we've collected enough
                if transactions.len() >= request.max_txs as usize {
                    break;
                }
            }
        }
        
        // Reverse to get topological order (parents before children)
        transactions.reverse();
        
        let has_more = transactions.len() >= request.max_txs as usize;
        let new_tips = self.dag.get_tips();
        
        Ok(SyncResponse {
            transactions,
            new_tips,
            has_more,
        })
    }
    
    /// Synchronize with a peer
    pub async fn sync_with_peer(&self, peer_tips: Vec<Hash>) -> Result<()> {
        let our_tips = self.dag.get_tips();
        
        // Find tips we don't have
        let missing_tips: Vec<Hash> = peer_tips.into_iter()
            .filter(|tip| !self.dag.contains_transaction(tip))
            .collect();
        
        if missing_tips.is_empty() {
            debug!("Already synchronized with peer");
            return Ok(());
        }
        
        info!("Need to sync {} missing tips", missing_tips.len());
        
        // Create sync request
        let request = SyncRequest {
            from_tips: missing_tips,
            max_txs: 1000,
            include_orphans: false,
        };
        
        // Add to sync queue
        self.sync_queue.write().await.push_back(request);
        
        Ok(())
    }
    
    /// Check if transaction should be propagated
    pub fn should_propagate(&self, tx: &Transaction) -> bool {
        match &self.propagation_strategy {
            PropagationStrategy::Flood => true,
            
            PropagationStrategy::RandomWalk { factor } => {
                // Probabilistic forwarding
                rand::random::<f64>() < *factor
            }
            
            PropagationStrategy::ValidatorsOnly => {
                // Only propagate validation-related transactions
                matches!(&tx.tx_type, 
                    dag_core::TransactionType::ValidatorUpdate { .. } |
                    dag_core::TransactionType::EntropyReductionClaim { .. }
                )
            }
            
            PropagationStrategy::EntropyWeighted { threshold } => {
                // Only propagate if transaction reduces enough entropy
                tx.is_entropy_reducing() && {
                    if let dag_core::TransactionType::EntropyReductionClaim { reduction, .. } = &tx.tx_type {
                        reduction.delta >= *threshold
                    } else {
                        false
                    }
                }
            }
        }
    }
    
    /// Get sync progress
    pub fn sync_progress(&self) -> f64 {
        // Simple heuristic: ratio of resolved to pending transactions
        let pending_count = self.pending_txs.try_read()
            .map(|p| p.len())
            .unwrap_or(0);
        
        let total_txs = 1000; // Would get from peer's reported DAG size
        
        if total_txs == 0 {
            1.0
        } else {
            1.0 - (pending_count as f64 / total_txs as f64)
        }
    }
}

/// Background sync worker
pub struct SyncWorker {
    sync_manager: Arc<SyncManager>,
    network_rx: mpsc::Receiver<P2PMessage>,
    network_tx: mpsc::Sender<P2PMessage>,
}

impl SyncWorker {
    pub fn new(
        sync_manager: Arc<SyncManager>,
        network_rx: mpsc::Receiver<P2PMessage>,
        network_tx: mpsc::Sender<P2PMessage>,
    ) -> Self {
        Self {
            sync_manager,
            network_rx,
            network_tx,
        }
    }
    
    pub async fn run(mut self) {
        info!("Sync worker started");
        
        while let Some(message) = self.network_rx.recv().await {
            match message {
                P2PMessage::NewTransaction { tx } => {
                    if let Err(e) = self.sync_manager.process_transaction(tx).await {
                        warn!("Failed to process transaction: {}", e);
                    }
                }
                
                P2PMessage::GetTransactions { hashes } => {
                    // Send requested transactions
                    let mut txs = Vec::new();
                    for hash in hashes {
                        if let Some(tx) = self.sync_manager.dag.get_transaction(&hash) {
                            txs.push((*tx).clone());
                        }
                    }
                    
                    if !txs.is_empty() {
                        let _ = self.network_tx.send(P2PMessage::Transactions { txs }).await;
                    }
                }
                
                P2PMessage::Tips { tips } => {
                    // Sync with peer's tips
                    if let Err(e) = self.sync_manager.sync_with_peer(tips).await {
                        warn!("Failed to sync with peer: {}", e);
                    }
                }
                
                _ => {}
            }
        }
    }
}