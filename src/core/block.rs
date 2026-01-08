use crate::core::transaction::Transaction;
use crate::crypto::Keypair;
use ed25519_dalek::{Signature, Verifier};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt;

/// Basic unit to store data of transaction
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Block {
    pub index: u64,
    pub previous_hash: String,
    pub timestamp: u64,
    pub transactions: Vec<Transaction>,
    pub hash: String,
    pub signature: Option<String>, 
    pub validator: Option<String>,
}

/// Data of block for hashing
#[derive(Serialize, Deserialize)]
struct BlockData {
    index: u64,
    previous_hash: String,
    timestamp: u64,
    transactions: Vec<Transaction>,
}

impl Block {
    pub fn new(index: u64, previous_hash: String, transactions: Vec<Transaction>) -> Self {
        let timestamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();

        let mut block = Self {
            index,
            previous_hash,
            timestamp,
            transactions,
            hash: String::new(),
            signature: None,
            validator: None,
        };

        block.hash = block.calc_hash();
        block
    }

    pub fn hash(&self) -> String {
        self.calc_hash()
    }

    fn calc_hash(&self) -> String {
        let mut hasher = Sha256::new();

        hasher.update(self.index.to_string().as_bytes());
        hasher.update(self.previous_hash.as_bytes());
        hasher.update(self.timestamp.to_string().as_bytes());

        for tx in &self.transactions {
            hasher.update(tx.to_bytes());
        }
        
        hex::encode(hasher.finalize())
    }

    pub fn sign(&mut self, keypair: &Keypair) -> Result<(), String> {
        let hash = self.hash();
        let message = hash.as_bytes();
        
        let signature = keypair.sign(message);
        
        self.signature = Some(hex::encode(signature.to_bytes()));
        self.validator = Some(keypair.public_key_hex());
        
        Ok(())
    }

    pub fn verify_signature(&self) -> bool {
        let signature_hex = match &self.signature {
            Some(sig) => sig,
            None => return false,
        };

        let validator_hex = match &self.validator {
            Some(v) => v,
            None => return false,
        };

        let signature_bytes = match hex::decode(signature_hex) {
            Ok(bytes) => bytes,
            Err(_) => return false,
        };

        if signature_bytes.len() != 64 {
            return false;
        }

        let signature = Signature::from_bytes(signature_bytes.as_slice().try_into().unwrap());

        let validator_bytes = match hex::decode(validator_hex) {
            Ok(bytes) => bytes,
            Err(_) => return false,
        };

        if validator_bytes.len() != 32 {
            return false;
        }

        let verifying_key = match ed25519_dalek::VerifyingKey::from_bytes(validator_bytes.as_slice().try_into().unwrap()) {
            Ok(key) => key,
            Err(_) => return false,
        };

        let hash = self.hash();
        let message = hash.as_bytes();
        verifying_key.verify(message, &signature).is_ok()
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        // Serialize without hash and signature for hashing
        let block_data = BlockData {
            index: self.index,
            previous_hash: self.previous_hash.clone(),
            timestamp: self.timestamp,
            transactions: self.transactions.clone(),
        };
        
        serde_json::to_string(&block_data).expect("Failed to serialize block").into_bytes()
    }
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Block({}, hash: {}, prev: {}, txs: {})", self.index, &self.hash[..16], &self.previous_hash[..16], self.transactions.len())
    }
}


