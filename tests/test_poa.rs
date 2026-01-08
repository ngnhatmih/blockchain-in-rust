use rustchain::consensus::poa::{get_validator_for_height, get_validator_public_key_for_height, is_valid_proposer};
use rustchain::consensus::validator::ValidatorSet;
use rustchain::core::Block;
use rustchain::core::Transaction;
use rustchain::crypto::Keypair;

#[test]
fn test_get_validator_for_height_round_robin() {
    let validator1 = Keypair::generate();
    let validator2 = Keypair::generate();
    let validator3 = Keypair::generate();

    let validator_keys = vec![validator1.public_key_hex(), validator2.public_key_hex(), validator3.public_key_hex()];

    let validator_set = ValidatorSet::new(validator_keys.clone());

    assert_eq!(get_validator_for_height(0, &validator_set), Some(0));
    assert_eq!(get_validator_public_key_for_height(0, &validator_set), Some(&validator_keys[0]));

    assert_eq!(get_validator_for_height(1, &validator_set), Some(1));
    assert_eq!(get_validator_public_key_for_height(1, &validator_set), Some(&validator_keys[1]));

    assert_eq!(get_validator_for_height(2, &validator_set), Some(2));
    assert_eq!(get_validator_public_key_for_height(2, &validator_set), Some(&validator_keys[2]));

    assert_eq!(get_validator_for_height(3, &validator_set), Some(0));
    assert_eq!(get_validator_public_key_for_height(3, &validator_set), Some(&validator_keys[0]));

    assert_eq!(get_validator_for_height(4, &validator_set), Some(1));
    assert_eq!(get_validator_public_key_for_height(4, &validator_set), Some(&validator_keys[1]));

    assert_eq!(get_validator_for_height(5, &validator_set), Some(2));
    assert_eq!(get_validator_public_key_for_height(5, &validator_set), Some(&validator_keys[2]));
}

#[test]
fn test_is_valid_proposer() {
    let validator1 = Keypair::generate();
    let validator2 = Keypair::generate();
    let validator3 = Keypair::generate();

    let validator_keys = vec![validator1.public_key_hex(), validator2.public_key_hex(), validator3.public_key_hex()];

    let validator_set = ValidatorSet::new(validator_keys.clone());

    let transactions = vec![Transaction::new("Alice".to_string(), "Bob".to_string(), 100)];
    let mut block0 = Block::new(0, "0".to_string(), transactions.clone());
    block0.sign(&validator1).unwrap();
    assert!(is_valid_proposer(&block0, &validator_set));

    let mut block0_wrong = Block::new(0, "0".to_string(), transactions.clone());
    block0_wrong.sign(&validator2).unwrap();
    assert!(!is_valid_proposer(&block0_wrong, &validator_set));

    let mut block5 = Block::new(5, "previous_hash".to_string(), transactions);
    block5.sign(&validator3).unwrap();
    assert!(is_valid_proposer(&block5, &validator_set));

    let mut block5_wrong = Block::new(5, "previous_hash".to_string(), vec![]);
    block5_wrong.sign(&validator1).unwrap();
    assert!(!is_valid_proposer(&block5_wrong, &validator_set));
}

#[test]
fn test_is_valid_proposer_no_validator() {
    let validator_set = ValidatorSet::new(vec![]);
    let block = Block::new(0, "0".to_string(), vec![]);
    assert!(!is_valid_proposer(&block, &validator_set));
}

