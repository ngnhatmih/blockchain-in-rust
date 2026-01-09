use libp2p::PeerId;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::core::chain::Blockchain;
use crate::core::block::Block;
use serde_json;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SyncState {
    Idle,
    GettingStatus,
    SyncingBlocks,
    Completed,
}

pub struct SyncManager {
    blockchain: Arc<Mutex<Blockchain>>,
    syncing_peer: Option<PeerId>,
    state: SyncState,
    target_height: u64,
    pending_blocks: HashMap<u64, bool>, // height -> is_pending
    batch_size: u64, // Number of blocks to request at once
}

impl SyncManager {
    pub fn new(blockchain: Arc<Mutex<Blockchain>>) -> Self {
        Self {
            blockchain,
            syncing_peer: None,
            state: SyncState::Idle,
            target_height: 0,
            pending_blocks: HashMap::new(),
            batch_size: 10, // Request 10 blocks at a time
        }
    }
    
    /// Start syncing from a peer
    pub fn start_sync(&mut self, peer: PeerId) {
        if self.state != SyncState::Idle {
            tracing::warn!("Sync already in progress, ignoring start_sync request");
            return;
        }
        
        self.syncing_peer = Some(peer);
        self.state = SyncState::GettingStatus;
        self.pending_blocks.clear();
        tracing::info!("Starting sync from peer {}", peer);
    }
    
    /// Check if we need to request status from peer
    pub fn needs_status_request(&self) -> bool {
        self.state == SyncState::GettingStatus
    }
    
    /// Process status response and determine what blocks to request
    pub async fn process_status_response(&mut self, peer_height: u64) -> Vec<(u64, u64)> {
        let local_height = {
            let chain = self.blockchain.lock().await;
            chain.len() as u64
        };
        
        let peer = self.syncing_peer.expect("syncing_peer should be set");
        tracing::info!("Peer {} has height {}, local height is {}", peer, peer_height, local_height);
        
        if peer_height <= local_height {
            tracing::info!("Already up to date or ahead of peer");
            self.state = SyncState::Completed;
            self.syncing_peer = None;
            return Vec::new();
        }
        
        self.target_height = peer_height;
        self.state = SyncState::SyncingBlocks;
        
        // Calculate which blocks we need
        self.calculate_missing_blocks(local_height, peer_height)
    }
    
    /// Calculate which blocks are missing and return ranges to request
    fn calculate_missing_blocks(&mut self, local_height: u64, target_height: u64) -> Vec<(u64, u64)> {
        let mut ranges = Vec::new();
        let mut start = local_height;
        
        while start < target_height {
            let end = (start + self.batch_size - 1).min(target_height - 1);
            ranges.push((start, end));
            
            // Mark blocks as pending
            for height in start..=end {
                self.pending_blocks.insert(height, true);
            }
            
            start = end + 1;
        }
        
        tracing::debug!("Calculated {} block ranges to request", ranges.len());
        ranges
    }
    
    /// Process a block response
    pub async fn process_block_response(&mut self, height: u64, block_data: Vec<u8>) -> anyhow::Result<bool> {
        // Deserialize block
        let block: Block = serde_json::from_slice(&block_data)
            .map_err(|e| anyhow::anyhow!("Failed to deserialize block: {}", e))?;
        
        // Validate block
        let mut chain = self.blockchain.lock().await;
        
        // Check if block already exists
        if chain.get_block_by_hash(&block.hash).is_some() {
            tracing::debug!("Block {} already exists, skipping", height);
            self.pending_blocks.remove(&height);
            return Ok(false);
        }
        
        // Validate and append
        if chain.validate_new_block(&block) {
            chain.append_block(block.clone())
                .map_err(|e| anyhow::anyhow!("Failed to append block: {}", e))?;
            tracing::info!("Synced block {} (hash: {})", height, &block.hash[..16]);
            self.pending_blocks.remove(&height);
            
            // Check if we're done
            let current_height = chain.len() as u64;
            if current_height >= self.target_height {
                self.state = SyncState::Completed;
                tracing::info!("Sync completed! Final height: {}", current_height);
                return Ok(true);
            }
            
            Ok(false)
        } else {
            tracing::warn!("Received invalid block {} from peer", height);
            self.pending_blocks.remove(&height);
            Err(anyhow::anyhow!("Invalid block received"))
        }
    }
    
    /// Process a blocks (multiple) response
    pub async fn process_blocks_response(&mut self, blocks_data: Vec<Vec<u8>>) -> anyhow::Result<bool> {
        for block_data in blocks_data {
            let block: Block = serde_json::from_slice(&block_data)
                .map_err(|e| anyhow::anyhow!("Failed to deserialize block: {}", e))?;
            
            let height = block.index;
            let mut chain = self.blockchain.lock().await;
            
            // Check if block already exists
            if chain.get_block_by_hash(&block.hash).is_some() {
                tracing::debug!("Block {} already exists, skipping", height);
                self.pending_blocks.remove(&height);
                continue;
            }
            
            // Validate and append
            if chain.validate_new_block(&block) {
                chain.append_block(block.clone())
                    .map_err(|e| anyhow::anyhow!("Failed to append block: {}", e))?;
                tracing::info!("Synced block {} (hash: {})", height, &block.hash[..16]);
                self.pending_blocks.remove(&height);
            } else {
                tracing::warn!("Received invalid block {} from peer", height);
                self.pending_blocks.remove(&height);
            }
        }
        
        // Check if we're done
        let current_height = {
            let chain = self.blockchain.lock().await;
            chain.len() as u64
        };
        
        if current_height >= self.target_height {
            self.state = SyncState::Completed;
            tracing::info!("Sync completed! Final height: {}", current_height);
            return Ok(true);
        }
        
        Ok(false)
    }
    
    /// Get the next block range to request (if any)
    pub async fn get_next_block_range(&self) -> Option<(u64, u64)> {
        if self.state != SyncState::SyncingBlocks {
            return None;
        }
        
        let local_height = {
            let chain = self.blockchain.lock().await;
            chain.len() as u64
        };
        
        if local_height >= self.target_height {
            return None;
        }
        
        let start = local_height;
        let end = (start + self.batch_size - 1).min(self.target_height - 1);
        
        // Check if we're already requesting this range
        let mut is_pending = false;
        for height in start..=end {
            if self.pending_blocks.get(&height).copied().unwrap_or(false) {
                is_pending = true;
                break;
            }
        }
        
        if is_pending {
            None // Wait for pending requests
        } else {
            Some((start, end))
        }
    }
    
    pub fn is_syncing(&self) -> bool {
        self.state != SyncState::Idle && self.state != SyncState::Completed
    }
    
    pub fn is_completed(&self) -> bool {
        self.state == SyncState::Completed
    }
    
    pub fn syncing_peer(&self) -> Option<PeerId> {
        self.syncing_peer
    }
    
    pub fn stop_sync(&mut self) {
        self.state = SyncState::Idle;
        self.syncing_peer = None;
        self.pending_blocks.clear();
        tracing::info!("Sync stopped");
    }
    
    pub fn reset(&mut self) {
        if self.state == SyncState::Completed {
            self.state = SyncState::Idle;
            self.syncing_peer = None;
            self.pending_blocks.clear();
        }
    }
}

