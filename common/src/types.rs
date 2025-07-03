use serde::{Deserialize, Serialize};
use std::fmt;

/// Core hash type used throughout the system
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Hash([u8; 32]);

impl Hash {
    pub fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn from_slice(slice: &[u8]) -> Result<Self, &'static str> {
        if slice.len() != 32 {
            return Err("Invalid hash length");
        }
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(slice);
        Ok(Self(bytes))
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn zero() -> Self {
        Self([0u8; 32])
    }
}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

/// Address type for accounts and contracts
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Address([u8; 20]);

impl Address {
    pub fn new(bytes: [u8; 20]) -> Self {
        Self(bytes)
    }

    pub fn from_public_key(public_key: &[u8]) -> Self {
        let hash = blake3::hash(public_key);
        let mut bytes = [0u8; 20];
        bytes.copy_from_slice(&hash.as_bytes()[..20]);
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{}", hex::encode(self.0))
    }
}

/// XP token representation with entropy tracking
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct XpAmount {
    /// Raw XP amount (scaled by 10^18 for precision)
    pub amount: u128,
    /// Associated entropy reduction in joules/kelvin
    pub entropy_delta: f64,
}

impl XpAmount {
    pub const DECIMALS: u32 = 18;
    
    pub fn new(amount: u128, entropy_delta: f64) -> Self {
        Self { amount, entropy_delta }
    }

    pub fn zero() -> Self {
        Self { amount: 0, entropy_delta: 0.0 }
    }
}

/// Timestamp with nanosecond precision
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Timestamp(u64);

impl Timestamp {
    pub fn now() -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        Self(duration.as_nanos() as u64)
    }

    pub fn as_nanos(&self) -> u64 {
        self.0
    }
}

/// Result type for operations that can fail
pub type Result<T> = std::result::Result<T, Error>;

/// Common error types
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid hash: {0}")]
    InvalidHash(String),
    
    #[error("Invalid signature: {0}")]
    InvalidSignature(String),
    
    #[error("Entropy validation failed: {0}")]
    EntropyValidation(String),
    
    #[error("DAG cycle detected: {0}")]
    CycleDetected(String),
    
    #[error("Transaction validation failed: {0}")]
    TransactionValidation(String),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    #[error("Storage error: {0}")]
    Storage(String),
    
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("Other error: {0}")]
    Other(String),
}