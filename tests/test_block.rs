use rustchain::core::{Block, Transaction};
use rustchain::crypto::Keypair;

#[test]
fn test_create_block() {
    let transactions = vec![
        Transaction::new("Alice".to_string(), "Bob".to_string(), 100),
    ];
    
    let block = Block::new(0, "0".to_string(), transactions);
    
    assert_eq!(block.index, 0);
    assert_eq!(block.previous_hash, "0");
    assert_eq!(block.transactions.len(), 1);
    assert!(!block.hash.is_empty());
}

#[test]
fn test_block_hash() {
    let transactions = vec![
        Transaction::new("Alice".to_string(), "Bob".to_string(), 100),
    ];
    
    let block = Block::new(0, "0".to_string(), transactions);
    let hash1 = block.hash();
    let hash2 = block.hash();

    assert_eq!(hash1, hash2);
    assert_eq!(hash1.len(), 64); 
}

#[test]
fn test_block_hash_different_blocks() {
    let tx1 = vec![Transaction::new("Alice".to_string(), "Bob".to_string(), 100)];
    let tx2 = vec![Transaction::new("Charlie".to_string(), "Dave".to_string(), 200)];
    
    let block1 = Block::new(0, "0".to_string(), tx1);
    let block2 = Block::new(0, "0".to_string(), tx2);

    // hash1 != hash2
    assert_ne!(block1.hash(), block2.hash());
}

#[test]
fn test_block_hash_depends_on_previous_hash() {
    let transactions = vec![
        Transaction::new("Alice".to_string(), "Bob".to_string(), 100),
    ];
    
    let block1 = Block::new(1, "hash1".to_string(), transactions.clone());
    let block2 = Block::new(1, "hash2".to_string(), transactions);
    
    // hash1 != hash2
    assert_ne!(block1.hash(), block2.hash());
}

#[test]
fn test_sign_block() {
    let keypair = Keypair::generate();
    let transactions = vec![
        Transaction::new("Alice".to_string(), "Bob".to_string(), 100),
    ];
    
    let mut block = Block::new(0, "0".to_string(), transactions);

    block.sign(&keypair).expect("Failed to sign block");
    
    assert!(block.signature.is_some());
    assert!(block.validator.is_some());
    assert_eq!(block.validator.as_ref().unwrap(), &keypair.public_key_hex());
}

#[test]
fn test_verify_signature() {
    let keypair = Keypair::generate();
    let transactions = vec![
        Transaction::new("Alice".to_string(), "Bob".to_string(), 100),
    ];
    
    let mut block = Block::new(0, "0".to_string(), transactions);

    block.sign(&keypair).expect("Could not sign block");

    assert!(block.verify_signature());
}

#[test]
fn test_verify_signature_fails_without_signature() {
    let transactions = vec![
        Transaction::new("Alice".to_string(), "Bob".to_string(), 100),
    ];
    
    let block = Block::new(0, "0".to_string(), transactions);

    assert!(!block.verify_signature());
}

#[test]
fn test_verify_signature_fails_with_tampered_block() {
    let keypair = Keypair::generate();
    let transactions = vec![
        Transaction::new("Alice".to_string(), "Bob".to_string(), 100),
    ];
    
    let mut block = Block::new(0, "0".to_string(), transactions);

    block.sign(&keypair).expect("Failed to sign block");

    block.transactions.push(
        Transaction::new("Eve".to_string(), "Mallory".to_string(), 999)
    );

    block.hash = block.hash();

    assert!(!block.verify_signature());
}

#[test]
fn test_block_serialize_deserialize() {
    let keypair = Keypair::generate();
    let transactions = vec![
        Transaction::new("Alice".to_string(), "Bob".to_string(), 100),
    ];
    
    let mut block1 = Block::new(0, "0".to_string(), transactions);
    block1.sign(&keypair).expect("Failed to sign block");
    
    // Serialize
    let json = serde_json::to_string(&block1).expect("Failed to serialize");
    
    // Deserialize
    let block2: Block = serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(block1.hash(), block2.hash());

    assert!(block2.verify_signature());
}

