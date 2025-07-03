use crate::transaction::Transaction;
use common::{Hash, Result, Error, XpTime};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};

/// DAG traversal algorithms for XP-Net

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DagTraversal {
    /// Maximum depth for traversal operations
    pub max_depth: usize,
    /// Include orphaned transactions
    pub include_orphans: bool,
    /// Temporal window for traversal
    pub temporal_window: Option<(XpTime, XpTime)>,
}

impl Default for DagTraversal {
    fn default() -> Self {
        Self {
            max_depth: 1000,
            include_orphans: false,
            temporal_window: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TraversalResult {
    /// Visited transactions in order
    pub visited: Vec<Hash>,
    /// Entropy flow through the path
    pub entropy_flow: f64,
    /// Causal relationships discovered
    pub causal_relationships: Vec<CausalRelationship>,
    /// Loop closures detected
    pub loop_closures: Vec<LoopClosure>,
    /// Temporal ordering
    pub temporal_ordering: Vec<(Hash, XpTime)>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CausalRelationship {
    /// Source transaction
    pub source: Hash,
    /// Target transaction
    pub target: Hash,
    /// Type of causal relationship
    pub relationship_type: RelationshipType,
    /// Strength of relationship (0.0 to 1.0)
    pub strength: f64,
    /// Entropy transfer amount
    pub entropy_transfer: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum RelationshipType {
    /// Direct parent-child relationship
    Direct,
    /// Indirect causal influence
    Indirect,
    /// Feedback loop relationship
    Feedback,
    /// Temporal dependency
    Temporal,
    /// Entropy flow relationship
    EntropyFlow,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoopClosure {
    /// Loop identifier
    pub loop_id: Hash,
    /// Transactions involved in the loop
    pub transactions: Vec<Hash>,
    /// Total entropy reduction
    pub entropy_reduction: f64,
    /// Loop closure time
    pub closure_time: XpTime,
    /// Loop type
    pub loop_type: String,
}

/// Interface for DAG traversal operations
pub trait DagTraversalProvider {
    /// Get transaction by hash
    fn get_transaction(&self, hash: &Hash) -> Option<Transaction>;
    
    /// Get children of a transaction
    fn get_children(&self, hash: &Hash) -> Vec<Hash>;
    
    /// Get parents of a transaction
    fn get_parents(&self, hash: &Hash) -> Vec<Hash>;
    
    /// Check if transaction exists
    fn contains(&self, hash: &Hash) -> bool;
}

impl DagTraversal {
    /// Perform breadth-first search from starting points
    pub fn bfs<T: DagTraversalProvider>(
        &self,
        provider: &T,
        start_points: Vec<Hash>,
    ) -> Result<TraversalResult> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut result = TraversalResult {
            visited: Vec::new(),
            entropy_flow: 0.0,
            causal_relationships: Vec::new(),
            loop_closures: Vec::new(),
            temporal_ordering: Vec::new(),
        };

        // Initialize queue with starting points
        for start in start_points {
            if provider.contains(&start) {
                queue.push_back((start, 0));
            }
        }

        while let Some((current_hash, depth)) = queue.pop_front() {
            if depth > self.max_depth || visited.contains(&current_hash) {
                continue;
            }

            visited.insert(current_hash);
            result.visited.push(current_hash);

            if let Some(current_tx) = provider.get_transaction(&current_hash) {
                // Add to temporal ordering
                result.temporal_ordering.push((current_hash, current_tx.xp_time));

                // Check temporal window
                if let Some((start_time, end_time)) = self.temporal_window {
                    if current_tx.xp_time < start_time || current_tx.xp_time > end_time {
                        continue;
                    }
                }

                // Calculate entropy contribution
                if current_tx.is_entropy_reducing() {
                    result.entropy_flow += self.extract_entropy_delta(&current_tx);
                }

                // Process children
                for child_hash in provider.get_children(&current_hash) {
                    if !visited.contains(&child_hash) {
                        queue.push_back((child_hash, depth + 1));
                        
                        // Record causal relationship
                        if let Some(child_tx) = provider.get_transaction(&child_hash) {
                            let relationship = self.analyze_relationship(&current_tx, &child_tx);
                            result.causal_relationships.push(relationship);
                        }
                    }
                }
            }
        }

        // Sort temporal ordering
        result.temporal_ordering.sort_by_key(|(_, time)| *time);

        // Detect loop closures
        result.loop_closures = self.detect_loops(provider, &result.visited)?;

        Ok(result)
    }

    /// Perform depth-first search for causal chains
    pub fn dfs_causal_chain<T: DagTraversalProvider>(
        &self,
        provider: &T,
        start: Hash,
        target: Hash,
    ) -> Result<Vec<Vec<Hash>>> {
        let mut chains = Vec::new();
        let mut current_path = Vec::new();
        let mut visited = HashSet::new();

        self.dfs_causal_recursive(
            provider,
            start,
            target,
            &mut current_path,
            &mut visited,
            &mut chains,
            0,
        )?;

        Ok(chains)
    }

    /// Recursive DFS helper for causal chains
    fn dfs_causal_recursive<T: DagTraversalProvider>(
        &self,
        provider: &T,
        current: Hash,
        target: Hash,
        path: &mut Vec<Hash>,
        visited: &mut HashSet<Hash>,
        chains: &mut Vec<Vec<Hash>>,
        depth: usize,
    ) -> Result<()> {
        if depth > self.max_depth {
            return Ok(());
        }

        path.push(current);
        visited.insert(current);

        if current == target {
            chains.push(path.clone());
        } else {
            for child in provider.get_children(&current) {
                if !visited.contains(&child) {
                    self.dfs_causal_recursive(provider, child, target, path, visited, chains, depth + 1)?;
                }
            }
        }

        path.pop();
        visited.remove(&current);

        Ok(())
    }

    /// Find entropy flow paths between transactions
    pub fn find_entropy_paths<T: DagTraversalProvider>(
        &self,
        provider: &T,
        from: Hash,
        to: Hash,
    ) -> Result<Vec<EntropyPath>> {
        let chains = self.dfs_causal_chain(provider, from, to)?;
        let mut entropy_paths = Vec::new();

        for chain in chains {
            let mut total_flow = 0.0;
            let mut entropy_events = Vec::new();

            for (i, &tx_hash) in chain.iter().enumerate() {
                if let Some(tx) = provider.get_transaction(&tx_hash) {
                    let entropy_delta = self.extract_entropy_delta(&tx);
                    if entropy_delta != 0.0 {
                        total_flow += entropy_delta;
                        entropy_events.push(EntropyEvent {
                            transaction: tx_hash,
                            entropy_delta,
                            timestamp: tx.xp_time,
                            position: i,
                        });
                    }
                }
            }

            if total_flow != 0.0 {
                entropy_paths.push(EntropyPath {
                    chain,
                    total_flow,
                    entropy_events,
                });
            }
        }

        // Sort by total entropy flow (highest first)
        entropy_paths.sort_by(|a, b| b.total_flow.partial_cmp(&a.total_flow).unwrap());

        Ok(entropy_paths)
    }

    /// Detect temporal loops in the transaction set
    fn detect_loops<T: DagTraversalProvider>(
        &self,
        provider: &T,
        transactions: &[Hash],
    ) -> Result<Vec<LoopClosure>> {
        let mut loops = Vec::new();
        let mut processed = HashSet::new();

        for &tx_hash in transactions {
            if processed.contains(&tx_hash) {
                continue;
            }

            if let Some(tx) = provider.get_transaction(&tx_hash) {
                // Look for loop closure patterns
                if let Some(loop_closure) = self.check_for_loop_closure(provider, &tx)? {
                    loops.push(loop_closure);
                    
                    // Mark all transactions in this loop as processed
                    for &loop_tx in &loop_closure.transactions {
                        processed.insert(loop_tx);
                    }
                }
            }
        }

        Ok(loops)
    }

    /// Check if a transaction represents a loop closure
    fn check_for_loop_closure<T: DagTraversalProvider>(
        &self,
        provider: &T,
        transaction: &Transaction,
    ) -> Result<Option<LoopClosure>> {
        use crate::transaction::TransactionType;

        match &transaction.tx_type {
            TransactionType::LoopClosure { loop_id, result, .. } => {
                if result.success {
                    // Find all transactions that contributed to this loop
                    let loop_transactions = self.find_loop_contributors(provider, transaction)?;
                    
                    let entropy_reduction = result.entropy_reduction
                        .as_ref()
                        .map(|r| r.delta)
                        .unwrap_or(0.0);

                    Some(LoopClosure {
                        loop_id: *loop_id,
                        transactions: loop_transactions,
                        entropy_reduction,
                        closure_time: transaction.xp_time,
                        loop_type: "general".to_string(),
                    })
                } else {
                    None
                }
            }

            TransactionType::TemporalLoop { temporal_loop, .. } => {
                if temporal_loop.is_closed() {
                    let entropy_reduction = temporal_loop.final_entropy
                        .map(|final_e| temporal_loop.initial_entropy - final_e)
                        .unwrap_or(0.0);

                    Some(LoopClosure {
                        loop_id: temporal_loop.id,
                        transactions: vec![transaction.hash],
                        entropy_reduction,
                        closure_time: transaction.xp_time,
                        loop_type: format!("{:?}", temporal_loop.loop_type),
                    })
                } else {
                    None
                }
            }

            _ => None,
        }
    }

    /// Find transactions that contributed to a loop
    fn find_loop_contributors<T: DagTraversalProvider>(
        &self,
        provider: &T,
        loop_closure_tx: &Transaction,
    ) -> Result<Vec<Hash>> {
        let mut contributors = Vec::new();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        // Start with the closure transaction's parents
        for parent in &loop_closure_tx.parents {
            queue.push_back(*parent);
        }

        while let Some(current_hash) = queue.pop_front() {
            if visited.contains(&current_hash) {
                continue;
            }
            visited.insert(current_hash);

            if let Some(tx) = provider.get_transaction(&current_hash) {
                // Check if this transaction contributes to the loop
                if self.contributes_to_loop(&tx, loop_closure_tx) {
                    contributors.push(current_hash);
                    
                    // Add parents to continue search
                    for parent in &tx.parents {
                        queue.push_back(*parent);
                    }
                }
            }
        }

        Ok(contributors)
    }

    /// Check if a transaction contributes to a specific loop
    fn contributes_to_loop(&self, tx: &Transaction, loop_closure: &Transaction) -> bool {
        // Simple heuristic: transactions from the same sender within a time window
        if let (Some(tx_sender), Some(loop_sender)) = (tx.sender(), loop_closure.sender()) {
            if tx_sender == loop_sender {
                // Check if transaction is within reasonable time window
                let time_diff = loop_closure.xp_time.to_total_minutes() - tx.xp_time.to_total_minutes();
                return time_diff <= 1440; // 24 hours
            }
        }
        
        false
    }

    /// Analyze relationship between two transactions
    fn analyze_relationship(&self, parent: &Transaction, child: &Transaction) -> CausalRelationship {
        let entropy_parent = self.extract_entropy_delta(parent);
        let entropy_child = self.extract_entropy_delta(child);
        let entropy_transfer = entropy_child; // Simplified

        let relationship_type = if child.parents.contains(&parent.hash) {
            RelationshipType::Direct
        } else if entropy_transfer.abs() > 0.1 {
            RelationshipType::EntropyFlow
        } else {
            let time_diff = child.xp_time.to_total_minutes() - parent.xp_time.to_total_minutes();
            if time_diff <= 60 { // Within 1 hour
                RelationshipType::Temporal
            } else {
                RelationshipType::Indirect
            }
        };

        let strength = match relationship_type {
            RelationshipType::Direct => 1.0,
            RelationshipType::EntropyFlow => (entropy_transfer.abs() / 10.0).min(1.0),
            RelationshipType::Temporal => 0.7,
            RelationshipType::Indirect => 0.3,
            RelationshipType::Feedback => 0.8,
        };

        CausalRelationship {
            source: parent.hash,
            target: child.hash,
            relationship_type,
            strength,
            entropy_transfer,
        }
    }

    /// Extract entropy delta from a transaction
    fn extract_entropy_delta(&self, tx: &Transaction) -> f64 {
        use crate::transaction::TransactionType;

        match &tx.tx_type {
            TransactionType::EntropyReductionClaim { reduction, .. } => reduction.delta,
            TransactionType::PhysicsXpMint { calculation_input, .. } => calculation_input.entropy_reduction,
            TransactionType::TemporalLoop { temporal_loop, .. } => temporal_loop.total_entropy_delta(),
            TransactionType::LoopClosure { result, .. } => {
                result.entropy_reduction.as_ref().map(|r| r.delta).unwrap_or(0.0)
            }
            _ => 0.0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EntropyPath {
    /// Chain of transactions forming the path
    pub chain: Vec<Hash>,
    /// Total entropy flow through the path
    pub total_flow: f64,
    /// Individual entropy events along the path
    pub entropy_events: Vec<EntropyEvent>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EntropyEvent {
    /// Transaction containing the entropy event
    pub transaction: Hash,
    /// Entropy change amount
    pub entropy_delta: f64,
    /// Timestamp of the event
    pub timestamp: XpTime,
    /// Position in the path
    pub position: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transaction::{Transaction, TransactionType, TransactionMetadata};
    use std::collections::HashMap;

    struct MockProvider {
        transactions: HashMap<Hash, Transaction>,
        children: HashMap<Hash, Vec<Hash>>,
    }

    impl MockProvider {
        fn new() -> Self {
            Self {
                transactions: HashMap::new(),
                children: HashMap::new(),
            }
        }

        fn add_transaction(&mut self, tx: Transaction) {
            let hash = tx.hash;
            for parent in &tx.parents {
                self.children.entry(*parent).or_default().push(hash);
            }
            self.transactions.insert(hash, tx);
        }
    }

    impl DagTraversalProvider for MockProvider {
        fn get_transaction(&self, hash: &Hash) -> Option<Transaction> {
            self.transactions.get(hash).cloned()
        }

        fn get_children(&self, hash: &Hash) -> Vec<Hash> {
            self.children.get(hash).cloned().unwrap_or_default()
        }

        fn get_parents(&self, hash: &Hash) -> Vec<Hash> {
            self.transactions.get(hash)
                .map(|tx| tx.parents.clone())
                .unwrap_or_default()
        }

        fn contains(&self, hash: &Hash) -> bool {
            self.transactions.contains_key(hash)
        }
    }

    #[test]
    fn test_bfs_traversal() {
        let mut provider = MockProvider::new();
        let traversal = DagTraversal::default();

        // Create a simple chain of transactions
        let tx1 = Transaction::new(
            TransactionType::Transfer {
                from: Address::new([1; 20]),
                to: Address::new([2; 20]),
                amount: common::XpAmount::new(1000, 0.0),
            },
            vec![],
            1,
            TransactionMetadata::default(),
        );

        let tx2 = Transaction::new(
            TransactionType::Transfer {
                from: Address::new([2; 20]),
                to: Address::new([3; 20]),
                amount: common::XpAmount::new(500, 0.0),
            },
            vec![tx1.hash],
            1,
            TransactionMetadata::default(),
        );

        provider.add_transaction(tx1.clone());
        provider.add_transaction(tx2.clone());

        let result = traversal.bfs(&provider, vec![tx1.hash]).unwrap();
        
        assert_eq!(result.visited.len(), 2);
        assert!(result.visited.contains(&tx1.hash));
        assert!(result.visited.contains(&tx2.hash));
        assert_eq!(result.causal_relationships.len(), 1);
    }

    #[test]
    fn test_entropy_path_detection() {
        let mut provider = MockProvider::new();
        let traversal = DagTraversal::default();

        // Create transactions with entropy reduction
        let entropy_reduction = common::EntropyReduction {
            initial: common::EntropyMeasurement {
                value: 5.0,
                confidence: 0.9,
                domain: "test".to_string(),
                method: common::MeasurementMethod::Information { bits: 100 },
            },
            final_state: common::EntropyMeasurement {
                value: 3.0,
                confidence: 0.9,
                domain: "test".to_string(),
                method: common::MeasurementMethod::Information { bits: 60 },
            },
            delta: 2.0,
            duration_ns: 1000000,
            validators: vec![],
        };

        let tx1 = Transaction::new(
            TransactionType::EntropyReductionClaim {
                claimant: Address::new([1; 20]),
                reduction: entropy_reduction,
            },
            vec![],
            1,
            TransactionMetadata::default(),
        );

        let tx2 = Transaction::new(
            TransactionType::Transfer {
                from: Address::new([1; 20]),
                to: Address::new([2; 20]),
                amount: common::XpAmount::new(1000, 0.0),
            },
            vec![tx1.hash],
            1,
            TransactionMetadata::default(),
        );

        provider.add_transaction(tx1.clone());
        provider.add_transaction(tx2.clone());

        let paths = traversal.find_entropy_paths(&provider, tx1.hash, tx2.hash).unwrap();
        
        assert!(!paths.is_empty());
        assert!(paths[0].total_flow > 0.0);
    }
}