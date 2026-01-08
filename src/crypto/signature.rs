use ed25519_dalek::{Signature, VerifyingKey, Verifier};

/// Verify signature for a message
pub fn verify_signature(verifying_key: &VerifyingKey, message: &[u8], signature: &Signature) -> bool {
    verifying_key.verify(message, signature).is_ok()
}


