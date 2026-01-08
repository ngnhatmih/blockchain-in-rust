use ed25519_dalek::{SigningKey, VerifyingKey, Signature, Signer};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Contains signing key and verifying key
#[derive(Clone, Debug)]
pub struct Keypair {
    signing_key: SigningKey,
    verifying_key: VerifyingKey,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SerializableKeypair {
    pub public_key: String,
    pub private_key: String,
}

impl Keypair {
    pub fn generate() -> Self {
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let verifying_key = signing_key.verifying_key();
        
        Self {
            signing_key,
            verifying_key,
        }
    }

    pub fn public_key_bytes(&self) -> [u8; 32] {
        self.verifying_key.to_bytes()
    }

    pub fn public_key_hex(&self) -> String {
        hex::encode(self.public_key_bytes())
    }
    pub fn sign(&self, message: &[u8]) -> Signature {
        self.signing_key.sign(message)
    }

    pub fn verifying_key(&self) -> &VerifyingKey {
        &self.verifying_key
    }

    pub fn from_seed(seed: &[u8; 32]) -> Self {
        let signing_key = SigningKey::from_bytes(seed);
        let verifying_key = signing_key.verifying_key();
        
        Self {
            signing_key,
            verifying_key,
        }
    }
}

impl Keypair {
    pub fn serialize(&self) -> Result<String, serde_json::Error> {
        let serializable = SerializableKeypair {
            public_key: self.public_key_hex(),
            private_key: hex::encode(self.signing_key.to_bytes()),
        };
        serde_json::to_string(&serializable)
    }

    pub fn deserialize(json: &str) -> Result<Self, String> {
        let serializable: SerializableKeypair = serde_json::from_str(json).map_err(|e| format!("Failed to parse JSON: {}", e))?;
        let private_key_bytes = hex::decode(&serializable.private_key).map_err(|e| format!("Failed to decode private key: {}", e))?;
        
        if private_key_bytes.len() != 32 {
            return Err("Invalid private key length".to_string());
        }

        let mut seed = [0u8; 32];
        seed.copy_from_slice(&private_key_bytes);
        
        Ok(Self::from_seed(&seed))
    }
}

impl fmt::Display for Keypair {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Keypair(pub: {})", self.public_key_hex())
    }
}


