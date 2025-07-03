use common::{Address, Hash, Signature, Timestamp, XpAmount, EntropyReduction, Result, Error, XpTime, TemporalLoop, PhysicsXpCalculator, XpCalculationInput};
use serde::{Deserialize, Serialize};

/// Transaction types in the DAG
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum TransactionType {
    /// XP transfer between addresses
    Transfer {
        from: Address,
        to: Address,
        amount: XpAmount,
    },
    /// Smart contract deployment
    ContractDeploy {
        deployer: Address,
        code_hash: Hash,
        init_params: Vec<u8>,
    },
    /// Smart contract invocation
    ContractCall {
        caller: Address,
        contract: Address,
        method: String,
        params: Vec<u8>,
        gas_limit: u64,
    },
    /// Entropy reduction claim (mints XP)
    EntropyReductionClaim {
        claimant: Address,
        reduction: EntropyReduction,
    },
    /// Validator registration/update
    ValidatorUpdate {
        validator: Address,
        stake: XpAmount,
        metadata: ValidatorMetadata,
    },
    /// Loop closure transaction
    LoopClosure {
        initiator: Address,
        loop_id: Hash,
        result: LoopResult,
    },
    /// Temporal loop transaction
    TemporalLoop {
        initiator: Address,
        temporal_loop: TemporalLoop,
    },
    /// Physics-based XP minting
    PhysicsXpMint {
        recipient: Address,
        calculation_input: XpCalculationInput,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ValidatorMetadata {
    pub endpoint: String,
    pub specializations: Vec<String>,
    pub reputation_score: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LoopResult {
    pub success: bool,
    pub entropy_reduction: Option<EntropyReduction>,
    pub evidence: Vec<u8>,
}

/// Complete transaction structure
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Transaction {
    /// Unique transaction hash
    pub hash: Hash,
    /// Transaction type and payload
    pub tx_type: TransactionType,
    /// References to parent transactions in the DAG
    pub parents: Vec<Hash>,
    /// Timestamp of transaction creation (Gregorian)
    pub timestamp: Timestamp,
    /// XP time of transaction (base-10 temporal system)
    pub xp_time: XpTime,
    /// Nonce for ordering multiple txs from same address
    pub nonce: u64,
    /// Signature of the transaction
    pub signature: Signature,
    /// Optional metadata
    pub metadata: TransactionMetadata,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct TransactionMetadata {
    /// Priority fee for faster inclusion
    pub priority_fee: Option<XpAmount>,
    /// Expiry timestamp
    pub expires_at: Option<Timestamp>,
    /// Domain-specific tags
    pub tags: Vec<String>,
}

impl Transaction {
    pub fn new(
        tx_type: TransactionType,
        parents: Vec<Hash>,
        nonce: u64,
        metadata: TransactionMetadata,
    ) -> Self {
        let timestamp = Timestamp::now();
        let xp_time = XpTime::now();
        let mut tx = Self {
            hash: Hash::zero(), // Will be computed
            tx_type,
            parents,
            timestamp,
            xp_time,
            nonce,
            signature: Signature::new(vec![]), // Will be set after signing
            metadata,
        };
        tx.hash = tx.compute_hash();
        tx
    }

    pub fn compute_hash(&self) -> Hash {
        use common::hash_all;
        
        let type_bytes = bincode::serialize(&self.tx_type).unwrap();
        let parents_bytes = bincode::serialize(&self.parents).unwrap();
        let timestamp_bytes = self.timestamp.as_nanos().to_le_bytes();
        let xp_time_bytes = bincode::serialize(&self.xp_time).unwrap();
        let nonce_bytes = self.nonce.to_le_bytes();
        
        hash_all(&[
            &type_bytes,
            &parents_bytes,
            &timestamp_bytes,
            &xp_time_bytes,
            &nonce_bytes,
        ])
    }

    pub fn sign(&mut self, private_key: &common::PrivateKey) {
        let message = self.signing_message();
        self.signature = private_key.sign(&message);
    }

    pub fn verify_signature(&self, public_key: &common::PublicKey) -> Result<()> {
        let message = self.signing_message();
        public_key.verify(&message, &self.signature)
    }

    fn signing_message(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(self.hash.as_bytes());
        data.extend_from_slice(&self.timestamp.as_nanos().to_le_bytes());
        data.extend_from_slice(&self.nonce.to_le_bytes());
        data
    }

    pub fn sender(&self) -> Option<Address> {
        match &self.tx_type {
            TransactionType::Transfer { from, .. } => Some(*from),
            TransactionType::ContractDeploy { deployer, .. } => Some(*deployer),
            TransactionType::ContractCall { caller, .. } => Some(*caller),
            TransactionType::EntropyReductionClaim { claimant, .. } => Some(*claimant),
            TransactionType::ValidatorUpdate { validator, .. } => Some(*validator),
            TransactionType::LoopClosure { initiator, .. } => Some(*initiator),
            TransactionType::TemporalLoop { initiator, .. } => Some(*initiator),
            TransactionType::PhysicsXpMint { recipient, .. } => Some(*recipient),
        }
    }

    pub fn is_entropy_reducing(&self) -> bool {
        match &self.tx_type {
            TransactionType::EntropyReductionClaim { reduction, .. } => reduction.delta > 0.0,
            TransactionType::LoopClosure { result, .. } => {
                result.success && result.entropy_reduction.is_some()
            }
            TransactionType::TemporalLoop { temporal_loop, .. } => {
                temporal_loop.total_entropy_delta() < 0.0 // Negative delta means entropy reduction
            }
            TransactionType::PhysicsXpMint { calculation_input, .. } => {
                calculation_input.entropy_reduction > 0.0
            }
            _ => false,
        }
    }

    pub fn validate_parents(&self, dag: &impl DagView) -> Result<()> {
        if self.parents.is_empty() && !dag.is_genesis(&self.hash) {
            return Err(Error::TransactionValidation("Transaction must have parents".into()));
        }

        for parent_hash in &self.parents {
            if !dag.contains_transaction(parent_hash) {
                return Err(Error::TransactionValidation(
                    format!("Parent transaction {} not found", parent_hash)
                ));
            }
        }

        Ok(())
    }
}

/// Interface for DAG queries needed by transactions
pub trait DagView {
    fn contains_transaction(&self, hash: &Hash) -> bool;
    fn is_genesis(&self, hash: &Hash) -> bool;
    fn get_transaction(&self, hash: &Hash) -> Option<Transaction>;
}