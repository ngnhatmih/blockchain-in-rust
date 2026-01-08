use rustchain::consensus::validator::ValidatorSet;
use rustchain::core::genesis::GenesisConfig;
use rustchain::core::Transaction;
use rustchain::crypto::Keypair;

#[test]
fn test_load_validators_from_genesis() {
    let validator1 = Keypair::generate();
    let validator2 = Keypair::generate();
    let validator3 = Keypair::generate();

    let validator_keys = vec![validator1.public_key_hex(), validator2.public_key_hex(), validator3.public_key_hex()];

    let transactions = vec![Transaction::new("Genesis".to_string(), "Alice".to_string(), 1000)];
    let config = GenesisConfig::new_with_validators(1234567890, transactions, validator_keys.clone());

    let validator_set = ValidatorSet::from_genesis(&config);

    assert_eq!(validator_set.len(), 3);
    assert_eq!(validator_set.get(0), Some(&validator_keys[0]));
    assert_eq!(validator_set.get(1), Some(&validator_keys[1]));
    assert_eq!(validator_set.get(2), Some(&validator_keys[2]));
}

#[test]
fn test_validator_set_contains() {
    let validator1 = Keypair::generate();
    let validator2 = Keypair::generate();
    let validator3 = Keypair::generate();

    let validator_keys = vec![validator1.public_key_hex(), validator2.public_key_hex(), validator3.public_key_hex()];

    let validator_set = ValidatorSet::new(validator_keys.clone());

    assert!(validator_set.contains(&validator_keys[0]));
    assert!(validator_set.contains(&validator_keys[1]));
    assert!(validator_set.contains(&validator_keys[2]));

    let non_validator = Keypair::generate();
    assert!(!validator_set.contains(&non_validator.public_key_hex()));
}

