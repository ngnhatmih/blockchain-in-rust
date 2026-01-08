use crate::core::Block;
use crate::consensus::validator::ValidatorSet;

pub fn get_validator_for_height(height: u64, validators: &ValidatorSet) -> Option<usize> {
    if validators.is_empty() {
        return None;
    }
    Some((height as usize) % validators.len())
}

pub fn get_validator_public_key_for_height(height: u64, validators: &ValidatorSet) -> Option<&String> {
    let index = get_validator_for_height(height, validators)?;
    validators.get(index)
}

pub fn is_valid_proposer(block: &Block, validators: &ValidatorSet) -> bool {
    let block_validator = match &block.validator {
        Some(v) => v,
        None => return false,
    };

    let expected_validator = match get_validator_public_key_for_height(block.index, validators) {
        Some(v) => v,
        None => return false,
    };

    block_validator == expected_validator
}

