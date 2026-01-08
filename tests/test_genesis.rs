use rustchain::core::genesis::{create_genesis_block, GenesisConfig};
use rustchain::core::Transaction;
use rustchain::crypto::Keypair;

#[test]
fn test_create_genesis_config() {
    let transactions = vec![Transaction::new("Genesis".to_string(), "Alice".to_string(), 1000)];
    let config = GenesisConfig::new(1234567890, transactions.clone());
    
    assert_eq!(config.timestamp, 1234567890);
    assert_eq!(config.transactions.len(), 1);
}

#[test]
fn test_genesis_config_serialize_deserialize() {
    let transactions = vec![Transaction::new("Genesis".to_string(), "Alice".to_string(), 1000)];
    let config1 = GenesisConfig::new(1234567890, transactions);
    
    let json = config1.to_json().expect("Failed to serialize");
    let config2 = GenesisConfig::from_json(&json).expect("Failed to deserialize");
    
    assert_eq!(config1.timestamp, config2.timestamp);
    assert_eq!(config1.transactions.len(), config2.transactions.len());
}

#[test]
fn test_create_genesis_block() {
    let transactions = vec![Transaction::new("Genesis".to_string(), "Alice".to_string(), 1000)];
    let config = GenesisConfig::new(1234567890, transactions);
    
    let block = create_genesis_block(&config, None);
    
    assert_eq!(block.index, 0);
    assert_eq!(block.previous_hash, "0");
    assert_eq!(block.timestamp, 1234567890);
    assert_eq!(block.transactions.len(), 1);
}

#[test]
fn test_create_genesis_block_with_validator() {
    let keypair = Keypair::generate();
    let transactions = vec![Transaction::new("Genesis".to_string(), "Alice".to_string(), 1000)];
    let config = GenesisConfig::new(1234567890, transactions);
    
    let block = create_genesis_block(&config, Some(&keypair));
    
    assert_eq!(block.index, 0);
    assert!(block.signature.is_some());
    assert!(block.validator.is_some());
    assert!(block.verify_signature());
}

