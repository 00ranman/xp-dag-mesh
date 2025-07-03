use crate::transaction::{Transaction, DagView};
use common::{Hash, Address, Result, Error, PhysicsXpCalculator, XpCalculationResult};
use dashmap::DashMap;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::algo::toposort;
use parking_lot::RwLock;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;

/// Main DAG structure
pub struct Dag {
    /// Transaction storage by hash
    transactions: Arc<DashMap<Hash, Arc<Transaction>>>,
    /// Graph structure for traversal
    graph: Arc<RwLock<DiGraph<Hash, ()>>>,
    /// Node index mapping
    node_indices: Arc<DashMap<Hash, NodeIndex>>,
    /// Tips of the DAG (transactions with no children)
    tips: Arc<RwLock<HashSet<Hash>>>,
    /// Genesis transactions
    genesis: Arc<RwLock<HashSet<Hash>>>,
    /// Account state tracking
    account_states: Arc<DashMap<Address, AccountState>>,
    /// Physics-based XP calculator
    xp_calculator: Arc<PhysicsXpCalculator>,
}

#[derive(Clone, Debug)]
pub struct AccountState {
    pub balance: u128,
    pub nonce: u64,
    pub entropy_contributed: f64,
    pub reputation_score: f64,
}

impl Dag {
    pub fn new() -> Self {
        Self {
            transactions: Arc::new(DashMap::new()),
            graph: Arc::new(RwLock::new(DiGraph::new())),
            node_indices: Arc::new(DashMap::new()),
            tips: Arc::new(RwLock::new(HashSet::new())),
            genesis: Arc::new(RwLock::new(HashSet::new())),
            account_states: Arc::new(DashMap::new()),
            xp_calculator: Arc::new(PhysicsXpCalculator::default()),
        }
    }

    pub fn add_genesis_transaction(&self, tx: Transaction) -> Result<()> {
        if !tx.parents.is_empty() {
            return Err(Error::TransactionValidation(
                "Genesis transaction cannot have parents".into()
            ));
        }

        let hash = tx.hash;
        self.add_transaction_internal(tx)?;
        
        let mut genesis = self.genesis.write();
        genesis.insert(hash);
        
        Ok(())
    }

    pub fn add_transaction(&self, tx: Transaction) -> Result<()> {
        // Validate parents exist
        tx.validate_parents(self)?;
        
        // Check for cycles
        if self.would_create_cycle(&tx) {
            return Err(Error::CycleDetected(
                format!("Transaction {} would create a cycle", tx.hash)
            ));
        }
        
        // Validate nonce
        if let Some(sender) = tx.sender() {
            let expected_nonce = self.get_account_nonce(&sender);
            if tx.nonce != expected_nonce {
                return Err(Error::TransactionValidation(
                    format!("Invalid nonce: expected {}, got {}", expected_nonce, tx.nonce)
                ));
            }
        }
        
        self.add_transaction_internal(tx)
    }

    fn add_transaction_internal(&self, tx: Transaction) -> Result<()> {
        let hash = tx.hash;
        
        // Add to transaction store
        self.transactions.insert(hash, Arc::new(tx.clone()));
        
        // Add to graph
        let mut graph = self.graph.write();
        let node_idx = graph.add_node(hash);
        self.node_indices.insert(hash, node_idx);
        
        // Connect to parents
        for parent_hash in &tx.parents {
            if let Some(parent_idx) = self.node_indices.get(parent_hash) {
                graph.add_edge(*parent_idx, node_idx, ());
                
                // Remove parent from tips
                let mut tips = self.tips.write();
                tips.remove(parent_hash);
            }
        }
        
        // Add to tips (will be removed when it becomes a parent)
        let mut tips = self.tips.write();
        tips.insert(hash);
        
        // Update account state
        self.process_transaction_effects(&tx)?;
        
        Ok(())
    }

    fn would_create_cycle(&self, tx: &Transaction) -> bool {
        let graph = self.graph.read();
        
        // Create temporary graph with new transaction
        let mut temp_graph = graph.clone();
        let temp_node = temp_graph.add_node(tx.hash);
        
        for parent_hash in &tx.parents {
            if let Some(parent_idx) = self.node_indices.get(parent_hash) {
                temp_graph.add_edge(*parent_idx, temp_node, ());
            }
        }
        
        // Check if topological sort is still possible
        toposort(&temp_graph, None).is_err()
    }

    fn process_transaction_effects(&self, tx: &Transaction) -> Result<()> {
        use crate::transaction::TransactionType;
        
        match &tx.tx_type {
            TransactionType::Transfer { from, to, amount } => {
                // Deduct from sender
                self.account_states.entry(*from).and_modify(|state| {
                    state.balance = state.balance.saturating_sub(amount.amount);
                    state.nonce += 1;
                });
                
                // Add to receiver
                self.account_states.entry(*to).and_modify(|state| {
                    state.balance = state.balance.saturating_add(amount.amount);
                }).or_insert(AccountState {
                    balance: amount.amount,
                    nonce: 0,
                    entropy_contributed: 0.0,
                    reputation_score: 0.0,
                });
            }
            
            TransactionType::EntropyReductionClaim { claimant, reduction } => {
                let xp_reward = reduction.calculate_xp();
                
                self.account_states.entry(*claimant).and_modify(|state| {
                    state.balance = state.balance.saturating_add(xp_reward);
                    state.entropy_contributed += reduction.delta;
                    state.nonce += 1;
                }).or_insert(AccountState {
                    balance: xp_reward,
                    nonce: 1,
                    entropy_contributed: reduction.delta,
                    reputation_score: 0.0,
                });
            }

            TransactionType::PhysicsXpMint { recipient, calculation_input } => {
                // Use the physics calculator to determine XP amount
                match self.xp_calculator.calculate_xp(calculation_input) {
                    Ok(xp_result) => {
                        self.account_states.entry(*recipient).and_modify(|state| {
                            state.balance = state.balance.saturating_add(xp_result.xp_amount);
                            state.entropy_contributed += calculation_input.entropy_reduction;
                            state.reputation_score = (state.reputation_score + calculation_input.reputation) / 2.0;
                            state.nonce += 1;
                        }).or_insert(AccountState {
                            balance: xp_result.xp_amount,
                            nonce: 1,
                            entropy_contributed: calculation_input.entropy_reduction,
                            reputation_score: calculation_input.reputation,
                        });
                    }
                    Err(_) => {
                        // Physics calculation failed - don't mint XP
                        if let Some(sender) = tx.sender() {
                            self.account_states.entry(sender).and_modify(|state| {
                                state.nonce += 1;
                            });
                        }
                    }
                }
            }

            TransactionType::TemporalLoop { initiator, temporal_loop } => {
                // Process temporal loop XP based on entropy reduction
                if let Some(entropy_reduction) = temporal_loop.final_entropy {
                    let total_reduction = temporal_loop.initial_entropy - entropy_reduction;
                    if total_reduction > 0.0 {
                        // Simple XP calculation for temporal loops
                        let xp_reward = (total_reduction * 1000.0) as u128;
                        
                        self.account_states.entry(*initiator).and_modify(|state| {
                            state.balance = state.balance.saturating_add(xp_reward);
                            state.entropy_contributed += total_reduction;
                            state.nonce += 1;
                        }).or_insert(AccountState {
                            balance: xp_reward,
                            nonce: 1,
                            entropy_contributed: total_reduction,
                            reputation_score: 0.0,
                        });
                    }
                }
            }
            
            _ => {
                // Update nonce for other transaction types
                if let Some(sender) = tx.sender() {
                    self.account_states.entry(sender).and_modify(|state| {
                        state.nonce += 1;
                    });
                }
            }
        }
        
        Ok(())
    }

    pub fn get_tips(&self) -> Vec<Hash> {
        self.tips.read().iter().copied().collect()
    }

    pub fn get_transaction(&self, hash: &Hash) -> Option<Arc<Transaction>> {
        self.transactions.get(hash).map(|entry| entry.clone())
    }

    pub fn get_account_state(&self, address: &Address) -> AccountState {
        self.account_states.get(address)
            .map(|entry| entry.clone())
            .unwrap_or(AccountState {
                balance: 0,
                nonce: 0,
                entropy_contributed: 0.0,
                reputation_score: 0.0,
            })
    }

    pub fn get_account_nonce(&self, address: &Address) -> u64 {
        self.get_account_state(address).nonce
    }

    /// Get transaction ancestors up to a certain depth
    pub fn get_ancestors(&self, hash: &Hash, max_depth: usize) -> HashSet<Hash> {
        let mut ancestors = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back((*hash, 0));
        
        while let Some((current_hash, depth)) = queue.pop_front() {
            if depth >= max_depth {
                continue;
            }
            
            if let Some(tx) = self.get_transaction(&current_hash) {
                for parent_hash in &tx.parents {
                    if ancestors.insert(*parent_hash) {
                        queue.push_back((*parent_hash, depth + 1));
                    }
                }
            }
        }
        
        ancestors
    }

    /// Get transaction descendants up to a certain depth
    pub fn get_descendants(&self, hash: &Hash, max_depth: usize) -> HashSet<Hash> {
        let mut descendants = HashSet::new();
        let graph = self.graph.read();
        
        if let Some(node_idx) = self.node_indices.get(hash) {
            let mut queue = VecDeque::new();
            queue.push_back((*node_idx, 0));
            
            while let Some((current_idx, depth)) = queue.pop_front() {
                if depth >= max_depth {
                    continue;
                }
                
                for edge in graph.edges(current_idx) {
                    let child_idx = edge.target();
                    if let Some(child_hash) = graph.node_weight(child_idx) {
                        if descendants.insert(*child_hash) {
                            queue.push_back((child_idx, depth + 1));
                        }
                    }
                }
            }
        }
        
        descendants
    }

    /// Perform topological sort of the DAG
    pub fn topological_sort(&self) -> Result<Vec<Hash>> {
        let graph = self.graph.read();
        
        match toposort(&*graph, None) {
            Ok(indices) => {
                let sorted_hashes: Vec<Hash> = indices
                    .into_iter()
                    .filter_map(|idx| graph.node_weight(idx).copied())
                    .collect();
                Ok(sorted_hashes)
            }
            Err(_) => Err(Error::CycleDetected("DAG contains a cycle".into())),
        }
    }
}

impl DagView for Dag {
    fn contains_transaction(&self, hash: &Hash) -> bool {
        self.transactions.contains_key(hash)
    }

    fn is_genesis(&self, hash: &Hash) -> bool {
        self.genesis.read().contains(hash)
    }

    fn get_transaction(&self, hash: &Hash) -> Option<Transaction> {
        self.transactions.get(hash).map(|entry| (*entry).clone())
    }
}