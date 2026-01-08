use rustchain::core::{Block, Blockchain, Transaction};
use rustchain::core::genesis::{create_genesis_block, GenesisConfig};

#[test]
fn test_create_blockchain_with_genesis() {
    let config = GenesisConfig::new(1234567890, vec![]);
    let genesis = create_genesis_block(&config, None);
    let chain = Blockchain::new(genesis);

    assert_eq!(chain.len(), 1);
    assert!(!chain.is_empty());
}

#[test]
fn test_append_block() {
    let config = GenesisConfig::new(1234567890, vec![]);
    let genesis = create_genesis_block(&config, None);
    let mut chain = Blockchain::new(genesis);

    let transactions = vec![Transaction::new("Alice".to_string(), "Bob".to_string(), 100)];
    let latest = chain.get_latest_block().unwrap();
    let new_block = Block::new(1, latest.hash.clone(), transactions);

    let result = chain.append_block(new_block);
    assert!(result.is_ok());
    assert_eq!(chain.len(), 2);
}

#[test]
fn test_get_block() {
    let config = GenesisConfig::new(1234567890, vec![]);
    let genesis = create_genesis_block(&config, None);
    let mut chain = Blockchain::new(genesis);

    let transactions = vec![Transaction::new("Alice".to_string(), "Bob".to_string(), 100)];
    let latest = chain.get_latest_block().unwrap();
    let new_block = Block::new(1, latest.hash.clone(), transactions);
    chain.append_block(new_block).unwrap();

    let block0 = chain.get_block(0);
    let block1 = chain.get_block(1);
    let block2 = chain.get_block(2);

    assert!(block0.is_some());
    assert_eq!(block0.unwrap().index, 0);
    assert!(block1.is_some());
    assert_eq!(block1.unwrap().index, 1);
    assert!(block2.is_none());
}

#[test]
fn test_get_block_by_hash() {
    let config = GenesisConfig::new(1234567890, vec![]);
    let genesis = create_genesis_block(&config, None);
    let chain = Blockchain::new(genesis);

    let genesis_block = chain.get_block(0).unwrap();
    let found = chain.get_block_by_hash(&genesis_block.hash);
    
    assert!(found.is_some());
    assert_eq!(found.unwrap().index, 0);
}

#[test]
fn test_validate_new_block() {
    let config = GenesisConfig::new(1234567890, vec![]);
    let genesis = create_genesis_block(&config, None);
    let chain = Blockchain::new(genesis);

    let transactions = vec![Transaction::new("Alice".to_string(), "Bob".to_string(), 100)];
    let latest = chain.get_latest_block().unwrap();
    let valid_block = Block::new(1, latest.hash.clone(), transactions.clone());
    let invalid_block = Block::new(2, latest.hash.clone(), transactions);

    assert!(chain.validate_new_block(&valid_block));
    assert!(!chain.validate_new_block(&invalid_block));
}

#[test]
fn test_validate_new_block_wrong_previous_hash() {
    let config = GenesisConfig::new(1234567890, vec![]);
    let genesis = create_genesis_block(&config, None);
    let chain = Blockchain::new(genesis);

    let transactions = vec![Transaction::new("Alice".to_string(), "Bob".to_string(), 100)];
    let invalid_block = Block::new(1, "wrong_hash".to_string(), transactions);

    assert!(!chain.validate_new_block(&invalid_block));
}

#[test]
fn test_append_five_blocks() {
    let config = GenesisConfig::new(1234567890, vec![]);
    let genesis = create_genesis_block(&config, None);
    let mut chain = Blockchain::new(genesis);

    for i in 1..=5 {
        let transactions = vec![Transaction::new(format!("User{}", i), format!("User{}", i + 1), 100)];
        let latest = chain.get_latest_block().unwrap();
        let new_block = Block::new(i, latest.hash.clone(), transactions);
        chain.append_block(new_block).expect("Could not append block");
    }

    assert_eq!(chain.len(), 6);
    assert!(chain.validate_chain());
}

#[test]
fn test_append_ten_blocks() {
    let config = GenesisConfig::new(1234567890, vec![]);
    let genesis = create_genesis_block(&config, None);
    let mut chain = Blockchain::new(genesis);

    for i in 1..=10 {
        let transactions = vec![Transaction::new(format!("User{}", i), format!("User{}", i + 1), 100)];
        let latest = chain.get_latest_block().unwrap();
        let new_block = Block::new(i, latest.hash.clone(), transactions);
        chain.append_block(new_block).expect("Could not append block");
    }

    assert_eq!(chain.len(), 11);
    assert!(chain.validate_chain());
    
    for i in 0..=10 {
        let block = chain.get_block(i).unwrap();
        assert_eq!(block.index, i as u64);
        if i > 0 {
            let prev_block = chain.get_block(i - 1).unwrap();
            assert_eq!(block.previous_hash, prev_block.hash);
        }
    }
}

#[test]
fn test_validate_chain() {
    let config = GenesisConfig::new(1234567890, vec![]);
    let genesis = create_genesis_block(&config, None);
    let mut chain = Blockchain::new(genesis);

    for i in 1..=3 {
        let transactions = vec![Transaction::new(format!("User{}", i), format!("User{}", i + 1), 100)];
        let latest = chain.get_latest_block().unwrap();
        let new_block = Block::new(i, latest.hash.clone(), transactions);
        chain.append_block(new_block).unwrap();
    }

    assert!(chain.validate_chain());
}

