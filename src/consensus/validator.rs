use crate::core::genesis::GenesisConfig;

/// List of chosen validators
#[derive(Clone, Debug)]
pub struct ValidatorSet {
    validators: Vec<String>, 
}

impl ValidatorSet {
    pub fn new(validators: Vec<String>) -> Self {
        Self { validators }
    }

    pub fn from_genesis(config: &GenesisConfig) -> Self {
        Self {
            validators: config.validators.clone(),
        }
    }

    pub fn len(&self) -> usize {
        self.validators.len()
    }

    pub fn is_empty(&self) -> bool {
        self.validators.is_empty()
    }

    pub fn get(&self, index: usize) -> Option<&String> {
        self.validators.get(index)
    }

    pub fn all(&self) -> &[String] {
        &self.validators
    }

    pub fn contains(&self, public_key: &str) -> bool {
        self.validators.contains(&public_key.to_string())
    }
}

