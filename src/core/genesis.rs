use crate::core::{Block, Transaction};
use crate::crypto::Keypair;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GenesisConfig {
    pub timestamp: u64,
    pub transactions: Vec<Transaction>,
    pub validators: Vec<String>, // List of validator public keys (hex encoded)
}

impl GenesisConfig {
    pub fn new(timestamp: u64, transactions: Vec<Transaction>) -> Self {
        Self { 
            timestamp, 
            transactions,
            validators: Vec::new(),
        }
    }

    pub fn new_with_validators(timestamp: u64, transactions: Vec<Transaction>, validators: Vec<String>) -> Self {
        Self { 
            timestamp, 
            transactions,
            validators,
        }
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

pub fn create_genesis_block(config: &GenesisConfig, validator: Option<&Keypair>) -> Block {
    let mut block = Block::new(0, "0".to_string(), config.transactions.clone());
    block.timestamp = config.timestamp;

    if let Some(keypair) = validator {
        block.sign(keypair).expect("Could not sign genesis block");
    }

    block
}