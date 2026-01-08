use rustchain::crypto::{Keypair, signature::verify_signature};

#[test]
fn test_verify_signature() {
    let keypair = Keypair::generate();
    let message = b"Test message";
    let signature = keypair.sign(message);
    
    assert!(verify_signature(keypair.verifying_key(), message, &signature));
}

