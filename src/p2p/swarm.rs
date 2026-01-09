use libp2p::{
    core::upgrade, identity, noise, tcp, yamux, Transport,
    Multiaddr, PeerId, Swarm, swarm::SwarmEvent,
};
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::behaviour::BlockchainBehaviour;
use super::config::P2PConfig;
use super::peer_manager::PeerManager;
use crate::network::protocols::{BlockSyncHandler, BlockAnnounceHandler};
use crate::network::sync::SyncManager;
use crate::core::chain::Blockchain;

pub struct P2PNetwork {
    pub swarm: Swarm<BlockchainBehaviour>,
    pub local_peer_id: PeerId,
    pub config: P2PConfig,
    pub peer_manager: Arc<Mutex<PeerManager>>,
    pub block_sync_handler: Option<BlockSyncHandler>,
    pub block_announce_handler: Option<BlockAnnounceHandler>,
    pub blockchain: Option<Arc<Mutex<Blockchain>>>,
    pub sync_manager: Option<Arc<Mutex<SyncManager>>>,
}

impl P2PNetwork {
    pub fn new(config: P2PConfig) -> Result<Self> {
        let local_key = identity::Keypair::generate_ed25519();
        let local_peer_id = PeerId::from(local_key.public());
        tracing::info!("Local peer id: {}", local_peer_id);

        // Build transport: TCP + Noise + Yamux
        let transport = tcp::tokio::Transport::default()
            .upgrade(upgrade::Version::V1Lazy)
            .authenticate(noise::Config::new(&local_key)?)
            .multiplex(yamux::Config::default())
            .boxed();

        // Create behaviour with all 4 protocols
        let behaviour = BlockchainBehaviour::new(
            local_peer_id, 
            &local_key,
            config.enable_mdns,
            config.heartbeat_interval,
        )?;

        // Use tokio executor for async runtime
        let swarm_config = libp2p::swarm::Config::with_tokio_executor()
            .with_idle_connection_timeout(std::time::Duration::from_secs(60));
        
        let swarm = Swarm::new(
            transport,
            behaviour,
            local_peer_id,
            swarm_config,
        );

        let peer_manager = Arc::new(Mutex::new(PeerManager::new()));

        Ok(Self {
            swarm,
            local_peer_id,
            config,
            peer_manager,
            block_sync_handler: None,
            block_announce_handler: None,
            blockchain: None,
            sync_manager: None,
        })
    }

    /// Initialize blockchain and protocol handlers
    pub fn with_blockchain(mut self, blockchain: Arc<Mutex<Blockchain>>) -> Self {
        self.blockchain = Some(blockchain.clone());
        self.block_sync_handler = Some(BlockSyncHandler::new(blockchain.clone()));
        self.block_announce_handler = Some(BlockAnnounceHandler::new());
        self.sync_manager = Some(Arc::new(Mutex::new(SyncManager::new(blockchain.clone()))));
        self
    }
    
    pub fn listen(&mut self) -> Result<()> {
        self.swarm.listen_on(self.config.listen_addr.clone())?;
        tracing::info!("Listening on {}", self.config.listen_addr);
        Ok(())
    }
    
    pub fn bootstrap(&mut self) -> Result<()> {
        for addr in &self.config.bootstrap_peers {
            match self.swarm.dial(addr.clone()) {
                Ok(()) => tracing::info!("Dialing {}", addr),
                Err(e) => tracing::warn!("Failed to dial {}: {}", addr, e),
            }
        }
        Ok(())
    }
    
    pub fn dial(&mut self, addr: Multiaddr) -> Result<()> {
        self.swarm.dial(addr)?;
        Ok(())
    }
    
    pub fn connected_peers(&self) -> Vec<PeerId> {
        self.swarm.connected_peers().cloned().collect()
    }

    pub fn peer_count(&self) -> usize {
        self.swarm.connected_peers().count()
    }

    /// Handle swarm events - should be called in event loop
    pub async fn handle_event(&mut self, event: SwarmEvent<super::behaviour::BlockchainBehaviourEvent>) {
        match event {
            SwarmEvent::Behaviour(behaviour_event) => {
                self.handle_behaviour_event(behaviour_event).await;
            }
            SwarmEvent::NewListenAddr { address, .. } => {
                tracing::info!("New listen address: {}", address);
            }
            SwarmEvent::ExpiredListenAddr { address, .. } => {
                tracing::warn!("Expired listen address: {}", address);
            }
            SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                tracing::info!("Connection established with peer: {}", peer_id);
                let mut pm = self.peer_manager.lock().await;
                pm.add_peer(peer_id, false);
                drop(pm);
                
                // Auto-start sync if not already syncing
                let should_start_sync = if let Some(ref sync_mgr) = self.sync_manager {
                    let mut sync = sync_mgr.lock().await;
                    if !sync.is_syncing() {
                        sync.start_sync(peer_id);
                        true
                    } else {
                        false
                    }
                } else {
                    false
                };
                
                // Request status to start sync (outside the lock)
                if should_start_sync {
                    if let Err(e) = self.send_block_sync_request(peer_id, crate::p2p::behaviour::BlockSyncRequest::GetStatus) {
                        tracing::warn!("Failed to request status for sync: {}", e);
                    }
                }
            }
            SwarmEvent::ConnectionClosed { peer_id, cause, .. } => {
                tracing::info!("Connection closed with peer: {} (cause: {:?})", peer_id, cause);
            }
            SwarmEvent::OutgoingConnectionError { error, .. } => {
                tracing::warn!("Outgoing connection error: {:?}", error);
            }
            SwarmEvent::IncomingConnectionError { error, .. } => {
                tracing::warn!("Incoming connection error: {:?}", error);
            }
            SwarmEvent::Dialing { peer_id, .. } => {
                if let Some(peer) = peer_id {
                    tracing::debug!("Dialing peer: {}", peer);
                } else {
                    tracing::debug!("Dialing unknown peer");
                }
            }
            _ => {}
        }
    }

    async fn handle_behaviour_event(&mut self, event: super::behaviour::BlockchainBehaviourEvent) {
        match event {
            super::behaviour::BlockchainBehaviourEvent::Ping(ping_event) => {
                match ping_event {
                    libp2p::ping::Event { peer, result, .. } => {
                        match result {
                            Ok(duration) => {
                                tracing::debug!("Ping to {}: {:?}", peer, duration);
                                let mut pm = self.peer_manager.lock().await;
                                pm.increase_reputation(&peer, 1);
                            }
                            Err(e) => {
                                tracing::warn!("Ping failed to {}: {:?}", peer, e);
                                let mut pm = self.peer_manager.lock().await;
                                pm.decrease_reputation(&peer, 5);
                            }
                        }
                    }
                }
            }
            super::behaviour::BlockchainBehaviourEvent::Identify(identify_event) => {
                match identify_event {
                    libp2p::identify::Event::Received { peer_id, info } => {
                        tracing::info!("Received identify info from {}: protocol={}", peer_id, info.protocol_version);
                    }
                    libp2p::identify::Event::Sent { .. } => {
                        tracing::debug!("Sent identify info");
                    }
                    libp2p::identify::Event::Error { peer_id, error } => {
                        tracing::warn!("Identify error with {}: {:?}", peer_id, error);
                    }
                    _ => {}
                }
            }
            super::behaviour::BlockchainBehaviourEvent::Gossipsub(gossipsub_event) => {
                match gossipsub_event {
                    libp2p::gossipsub::Event::Message { message, .. } => {
                        if let Some(source) = message.source {
                            tracing::debug!("Received gossipsub message from {}", source);
                        } else {
                            tracing::debug!("Received gossipsub message from unknown source");
                        }
                        
                        // Handle block announcement via gossip
                        if let Some(ref handler) = self.block_announce_handler {
                            if let Some(ref blockchain) = self.blockchain {
                                if let Err(e) = handler.handle_message(message, blockchain.clone()).await {
                                    tracing::warn!("Failed to handle gossip message: {}", e);
                                }
                            }
                        }
                    }
                    libp2p::gossipsub::Event::Subscribed { peer_id, topic } => {
                        tracing::info!("Peer {} subscribed to topic {}", peer_id, topic);
                    }
                    libp2p::gossipsub::Event::Unsubscribed { peer_id, topic } => {
                        tracing::info!("Peer {} unsubscribed from topic {}", peer_id, topic);
                    }
                    _ => {}
                }
            }
            super::behaviour::BlockchainBehaviourEvent::BlockSync(sync_event) => {
                match sync_event {
                    libp2p::request_response::Event::Message { message, .. } => {
                        match message {
                            libp2p::request_response::Message::Request { request, channel, .. } => {
                                // Handle incoming request
                                if let Some(ref handler) = self.block_sync_handler {
                                    if let Err(e) = handler.handle_request(request, channel, &mut self.swarm).await {
                                        tracing::warn!("Failed to handle block sync request: {}", e);
                                    }
                                } else {
                                    tracing::warn!("Received block sync request but no handler is configured");
                                }
                            }
                            libp2p::request_response::Message::Response { response, request_id, .. } => {
                                tracing::debug!("Received block sync response (request_id: {:?})", request_id);
                                
                                // Handle response in sync manager
                                // We need to collect all data first, then drop locks before sending new requests
                                let mut ranges_to_request = Vec::new();
                                let mut next_range_to_request = None;
                                
                                if let Some(ref sync_mgr) = self.sync_manager {
                                    let mut sync = sync_mgr.lock().await;
                                    
                                    match response {
                                        crate::p2p::behaviour::BlockSyncResponse::Status { height } => {
                                            if sync.needs_status_request() {
                                                let ranges = sync.process_status_response(height).await;
                                                let peer = sync.syncing_peer();
                                                drop(sync);
                                                
                                                // Collect ranges to request
                                                if let Some(peer) = peer {
                                                    ranges_to_request = ranges.into_iter().map(|(s, e)| (peer, s, e)).collect();
                                                }
                                            }
                                        }
                                        crate::p2p::behaviour::BlockSyncResponse::Block(_block_data) => {
                                            // We need to know which height this is for
                                            // For now, we'll need to track this differently
                                            // This is a limitation - we should track request_id -> height mapping
                                            tracing::warn!("Received single block response but height is unknown");
                                        }
                                        crate::p2p::behaviour::BlockSyncResponse::Blocks(blocks_data) => {
                                            let peer = sync.syncing_peer();
                                            
                                            // Process blocks and check if we need more
                                            let (completed, next_range) = match sync.process_blocks_response(blocks_data).await {
                                                Ok(completed) => {
                                                    if completed {
                                                        sync.reset();
                                                        (true, None)
                                                    } else {
                                                        (false, sync.get_next_block_range().await)
                                                    }
                                                }
                                                Err(e) => {
                                                    tracing::warn!("Failed to process blocks response: {}", e);
                                                    (false, None)
                                                }
                                            };
                                            
                                            drop(sync);
                                            
                                            // Collect next range to request if needed
                                            if !completed {
                                                if let (Some(peer), Some((start, end))) = (peer, next_range) {
                                                    next_range_to_request = Some((peer, start, end));
                                                }
                                            }
                                        }
                                        crate::p2p::behaviour::BlockSyncResponse::NotFound => {
                                            tracing::warn!("Peer responded with NotFound");
                                        }
                                    }
                                }
                                
                                // Now send requests (outside of any locks)
                                for (peer, start, end) in ranges_to_request {
                                    let request = crate::p2p::behaviour::BlockSyncRequest::GetBlocks { start, end };
                                    if let Err(e) = self.send_block_sync_request(peer, request) {
                                        tracing::warn!("Failed to request blocks {}-{}: {}", start, end, e);
                                    } else {
                                        tracing::debug!("Requested blocks {}-{} from peer {}", start, end, peer);
                                    }
                                }
                                
                                if let Some((peer, start, end)) = next_range_to_request {
                                    let request = crate::p2p::behaviour::BlockSyncRequest::GetBlocks { start, end };
                                    if let Err(e) = self.send_block_sync_request(peer, request) {
                                        tracing::warn!("Failed to request next batch {}-{}: {}", start, end, e);
                                    } else {
                                        tracing::debug!("Requested next batch {}-{} from peer {}", start, end, peer);
                                    }
                                }
                            }
                        }
                    }
                    libp2p::request_response::Event::OutboundFailure { error, .. } => {
                        tracing::warn!("Block sync outbound failure: {:?}", error);
                    }
                    libp2p::request_response::Event::InboundFailure { error, .. } => {
                        tracing::warn!("Block sync inbound failure: {:?}", error);
                    }
                    libp2p::request_response::Event::ResponseSent { .. } => {
                        tracing::debug!("Block sync response sent");
                    }
                }
            }
            super::behaviour::BlockchainBehaviourEvent::Mdns(mdns_event) => {
                if !self.config.enable_mdns {
                    return; // Ignore mDNS events if disabled
                }
                match mdns_event {
                    libp2p::mdns::Event::Discovered(list) => {
                        for (peer_id, multiaddr) in list {
                            tracing::info!("mDNS discovered peer: {} at {}", peer_id, multiaddr);
                            // Auto-dial discovered peers if under max_peers limit
                            if self.peer_count() < self.config.max_peers {
                                // Store for dialing in event loop (can't dial here as swarm is borrowed)
                                tracing::debug!("Will dial mDNS discovered peer: {} at {}", peer_id, multiaddr);
                            }
                        }
                    }
                    libp2p::mdns::Event::Expired(list) => {
                        for (peer_id, _) in list {
                            tracing::debug!("mDNS peer expired: {}", peer_id);
                        }
                    }
                }
            }
        }
    }

    /// Get a reference to the swarm for polling events
    pub fn swarm_mut(&mut self) -> &mut Swarm<BlockchainBehaviour> {
        &mut self.swarm
    }

    /// Announce a block via gossip protocol
    pub fn announce_block(&mut self, block: &crate::core::block::Block) -> anyhow::Result<()> {
        if let Some(ref handler) = self.block_announce_handler {
            handler.announce_block(block, &mut self.swarm)
        } else {
            Err(anyhow::anyhow!("Block announce handler not initialized"))
        }
    }

    /// Send a block sync request to a peer
    pub fn send_block_sync_request(
        &mut self,
        peer: PeerId,
        request: crate::p2p::behaviour::BlockSyncRequest,
    ) -> anyhow::Result<()> {
        if let Some(ref handler) = self.block_sync_handler {
            handler.send_request(peer, request, &mut self.swarm);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Block sync handler not initialized"))
        }
    }
}
