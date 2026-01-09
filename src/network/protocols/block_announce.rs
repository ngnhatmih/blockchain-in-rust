use libp2p::{gossipsub::{IdentTopic, Message}, swarm::Swarm};
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::p2p::behaviour::BlockchainBehaviour;
use crate::core::block::Block;
use crate::core::chain::Blockchain;
use serde_json;

pub struct BlockAnnounceHandler {
    block_topic: IdentTopic,
}

impl BlockAnnounceHandler {
    pub fn new() -> Self {
        Self {
            block_topic: IdentTopic::new("blocks"),
        }
    }
    
    pub fn announce_block(
        &self,
        block: &Block,
        swarm: &mut Swarm<BlockchainBehaviour>,
    ) -> anyhow::Result<()> {
        let data = serde_json::to_vec(block)
            .map_err(|e| anyhow::anyhow!("Failed to serialize block: {}", e))?;
        
        swarm.behaviour_mut()
            .gossipsub
            .publish(self.block_topic.clone(), data)?;
        
        tracing::debug!("Announced block {}", block.index);
        Ok(())
    }
    
    pub async fn handle_message(
        &self,
        message: Message,
        blockchain: Arc<Mutex<Blockchain>>,
    ) -> anyhow::Result<()> {
        let block: Block = serde_json::from_slice(&message.data)
            .map_err(|e| anyhow::anyhow!("Failed to deserialize block: {}", e))?;
        
        let mut chain = blockchain.lock().await;
        
        // Check if block already exists
        if chain.get_block_by_hash(&block.hash).is_some() {
            tracing::debug!("Received duplicate block {} via gossip (hash: {})", block.index, &block.hash[..16]);
            return Ok(());
        }
        
        // Validate and append block
        if chain.validate_new_block(&block) {
            chain.append_block(block.clone())
                .map_err(|e| anyhow::anyhow!("Failed to append block: {}", e))?;
            tracing::info!("Received and added block {} via gossip (hash: {})", block.index, &block.hash[..16]);
        } else {
            tracing::warn!("Received invalid block {} via gossip (hash: {})", block.index, &block.hash[..16]);
        }
        
        Ok(())
    }
}
