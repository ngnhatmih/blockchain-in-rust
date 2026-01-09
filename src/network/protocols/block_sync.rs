use libp2p::{request_response::ResponseChannel, PeerId, swarm::Swarm};
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::p2p::behaviour::{BlockchainBehaviour, BlockSyncRequest, BlockSyncResponse};
use crate::core::chain::Blockchain;
use serde_json;

pub struct BlockSyncHandler {
    blockchain: Arc<Mutex<Blockchain>>,
}

impl BlockSyncHandler {
    pub fn new(blockchain: Arc<Mutex<Blockchain>>) -> Self {
        Self { blockchain }
    }
    
    pub async fn handle_request(
        &self,
        request: BlockSyncRequest,
        channel: ResponseChannel<BlockSyncResponse>,
        swarm: &mut Swarm<BlockchainBehaviour>,
    ) -> anyhow::Result<()> {
        let response = match request {
            BlockSyncRequest::GetBlock { height } => {
                tracing::debug!("Handling GetBlock request for height {}", height);
                let chain = self.blockchain.lock().await;
                if let Some(block) = chain.get_block(height as usize) {
                    let serialized = serde_json::to_vec(block)
                        .map_err(|e| anyhow::anyhow!("Failed to serialize block: {}", e))?;
                    tracing::debug!("Sending block {} in response", height);
                    BlockSyncResponse::Block(serialized)
                } else {
                    tracing::debug!("Block at height {} not found", height);
                    BlockSyncResponse::NotFound
                }
            }
            BlockSyncRequest::GetBlocks { start, end } => {
                tracing::debug!("Handling GetBlocks request for range {}..={}", start, end);
                let chain = self.blockchain.lock().await;
                let mut blocks = Vec::new();
                for height in start..=end {
                    if let Some(block) = chain.get_block(height as usize) {
                        let serialized = serde_json::to_vec(block)
                            .map_err(|e| anyhow::anyhow!("Failed to serialize block: {}", e))?;
                        blocks.push(serialized);
                    }
                }
                tracing::debug!("Sending {} blocks in response", blocks.len());
                BlockSyncResponse::Blocks(blocks)
            }
            BlockSyncRequest::GetStatus => {
                tracing::debug!("Handling GetStatus request");
                let chain = self.blockchain.lock().await;
                let height = chain.len() as u64;
                tracing::debug!("Sending status with height {}", height);
                BlockSyncResponse::Status { height }
            }
        };

        swarm.behaviour_mut()
            .block_sync
            .send_response(channel, response)
            .map_err(|e| anyhow::anyhow!("Failed to send response: {:?}", e))?;
        
        Ok(())
    }
    
    pub fn send_request(
        &self,
        peer: PeerId,
        request: BlockSyncRequest,
        swarm: &mut Swarm<BlockchainBehaviour>,
    ) {
        swarm.behaviour_mut()
            .block_sync
            .send_request(&peer, request);
    }
}
