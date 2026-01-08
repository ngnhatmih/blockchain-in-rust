use rustchain::crypto::Keypair;
use ed25519_dalek::Verifier;

#[test]
fn test_generate_keypair() {
    let keypair = Keypair::generate();
    assert_eq!(keypair.public_key_bytes().len(), 32);
}

#[test]
fn test_sign_and_verify() {
    let keypair = Keypair::generate();
    let message = b"Hello, Blockchain!";
    
    // Sign
    let signature = keypair.sign(message);
    
    // Verify
    let verifying_key = keypair.verifying_key();
    assert!(verifying_key.verify(message, &signature).is_ok());
}

#[test]
fn test_sign_and_verify_fails_with_wrong_message() {
    let keypair = Keypair::generate();
    let message = b"Hello, Blockchain!";
    let wrong_message = b"Hello, Wrong!";
    
    let signature = keypair.sign(message);
    let verifying_key = keypair.verifying_key();

    assert!(verifying_key.verify(wrong_message, &signature).is_err());
}

#[test]
fn test_serialize_and_deserialize() {
    let keypair1 = Keypair::generate();
    let message = b"Test message";
    let signature1 = keypair1.sign(message);
    
    // Serialize
    let json = keypair1.serialize().expect("Failed to serialize");
    
    // Deserialize
    let keypair2 = Keypair::deserialize(&json).expect("Failed to deserialize");

    let verifying_key2 = keypair2.verifying_key();
    assert!(verifying_key2.verify(message, &signature1).is_ok());

    assert_eq!(keypair1.public_key_hex(), keypair2.public_key_hex());
}

#[test]
fn test_from_seed_deterministic() {
    let seed = [42u8; 32];
    let keypair1 = Keypair::from_seed(&seed);
    let keypair2 = Keypair::from_seed(&seed);
    
    // keypair1 == keypair2
    assert_eq!(keypair1.public_key_hex(), keypair2.public_key_hex());
}

