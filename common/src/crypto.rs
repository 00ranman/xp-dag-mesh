use crate::types::{Hash, Result, Error};
use ed25519_dalek::{Signature as Ed25519Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};

/// Digital signature
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Signature(Vec<u8>);

impl Signature {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// Public key for signature verification
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PublicKey(Vec<u8>);

impl PublicKey {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn verify(&self, message: &[u8], signature: &Signature) -> Result<()> {
        let verifying_key = VerifyingKey::from_bytes(
            self.0.as_slice().try_into()
                .map_err(|_| Error::InvalidSignature("Invalid public key length".into()))?
        ).map_err(|e| Error::InvalidSignature(e.to_string()))?;

        let sig = Ed25519Signature::from_bytes(
            signature.as_bytes().try_into()
                .map_err(|_| Error::InvalidSignature("Invalid signature length".into()))?
        );

        verifying_key.verify(message, &sig)
            .map_err(|e| Error::InvalidSignature(e.to_string()))
    }
}

/// Private key for signing
pub struct PrivateKey(SigningKey);

impl PrivateKey {
    pub fn generate() -> Self {
        use rand::rngs::OsRng;
        Self(SigningKey::generate(&mut OsRng))
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let key = SigningKey::from_bytes(
            bytes.try_into()
                .map_err(|_| Error::InvalidSignature("Invalid private key length".into()))?
        );
        Ok(Self(key))
    }

    pub fn public_key(&self) -> PublicKey {
        PublicKey(self.0.verifying_key().to_bytes().to_vec())
    }

    pub fn sign(&self, message: &[u8]) -> Signature {
        let sig = self.0.sign(message);
        Signature(sig.to_bytes().to_vec())
    }
}

/// Hash function wrapper
pub fn hash(data: &[u8]) -> Hash {
    let hash = blake3::hash(data);
    Hash::new(*hash.as_bytes())
}

/// Hash multiple pieces of data
pub fn hash_all(data: &[&[u8]]) -> Hash {
    let mut hasher = blake3::Hasher::new();
    for d in data {
        hasher.update(d);
    }
    Hash::new(*hasher.finalize().as_bytes())
}