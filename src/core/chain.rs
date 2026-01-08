use crate::core::Block;
use std::collections::HashMap;


/// Blockchain is set of blocks that is linked together by hash of previous block
pub struct Blockchain {
    blocks: Vec<Block>,
    block_map: HashMap<String, usize>,
}

impl Blockchain {
    pub fn new(genesis_block: Block) -> Self {
        let mut chain = Self {
            blocks: vec![genesis_block.clone()],
            block_map: HashMap::new(),
        };
        chain.block_map.insert(genesis_block.hash.clone(), 0);
        chain
    }

    pub fn append_block(&mut self, block: Block) -> Result<(), String> {
        if !self.validate_new_block(&block) {
            return Err("Block validation failed".to_string());
        }

        let index = self.blocks.len();
        self.block_map.insert(block.hash.clone(), index);
        self.blocks.push(block);
        Ok(())
    }

    pub fn get_block(&self, index: usize) -> Option<&Block> {
        self.blocks.get(index)
    }

    pub fn get_block_by_hash(&self, hash: &str) -> Option<&Block> {
        self.block_map.get(hash).and_then(|&idx| self.blocks.get(idx))
    }

    pub fn validate_new_block(&self, block: &Block) -> bool {
        if block.index != self.blocks.len() as u64 {
            return false;
        }

        let previous_block = match self.blocks.last() {
            Some(b) => b,
            None => return false,
        };

        if block.previous_hash != previous_block.hash {
            return false;
        }

        if block.hash() != block.hash {
            return false;
        }

        true
    }

    pub fn len(&self) -> usize {
        self.blocks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.blocks.is_empty()
    }

    pub fn get_latest_block(&self) -> Option<&Block> {
        self.blocks.last()
    }

    pub fn validate_chain(&self) -> bool {
        if self.blocks.is_empty() {
            return false;
        }

        for i in 1..self.blocks.len() {
            let current = &self.blocks[i];
            let previous = &self.blocks[i - 1];

            if current.index != i as u64 {
                return false;
            }

            if current.previous_hash != previous.hash {
                return false;
            }

            if current.hash() != current.hash {
                return false;
            }
        }

        true
    }
}
