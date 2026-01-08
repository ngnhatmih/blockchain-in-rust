use serde::{Deserialize, Serialize};
use std::fmt;

/// Transaction data contains sender, receiver and amount
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub amount: u64,
}

impl Transaction {
    pub fn new(from: String, to: String, amount: u64) -> Self {
        Self { from, to, amount }
    }

    pub fn serialize(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn deserialize(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.serialize().expect("Failed to serialize transaction").into_bytes()
    }
}

impl fmt::Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Tx({} -> {}: {})", self.from, self.to, self.amount)
    }
}


