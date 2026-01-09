use libp2p::PeerId;
use std::collections::{HashMap, HashSet};
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct PeerInfo {
    pub peer_id: PeerId,
    pub reputation: i32,
    pub misbehavior_count: u32,
    pub last_seen: Instant,
    pub is_validator: bool,
}

pub struct PeerManager {
    peers: HashMap<PeerId, PeerInfo>,
    blacklist: HashSet<PeerId>,
    validators: HashSet<PeerId>,
    ban_threshold: i32,
    unban_threshold: i32,
}

impl PeerManager {
    pub fn new() -> Self {
        Self {
            peers: HashMap::new(),
            blacklist: HashSet::new(),
            validators: HashSet::new(),
            ban_threshold: -100,
            unban_threshold: -50,
        }
    }
    
    pub fn add_peer(&mut self, peer_id: PeerId, is_validator: bool) {
        let peer_info = self.peers.entry(peer_id).or_insert_with(|| PeerInfo {
            peer_id,
            reputation: 0,
            misbehavior_count: 0,
            last_seen: Instant::now(),
            is_validator,
        });
        peer_info.last_seen = Instant::now();
        if is_validator {
            self.validators.insert(peer_id);
            peer_info.is_validator = true;
        }
    }
    
    pub fn remove_peer(&mut self, peer_id: &PeerId) {
        self.peers.remove(peer_id);
        self.validators.remove(peer_id);
    }
    
    pub fn increase_reputation(&mut self, peer_id: &PeerId, delta: i32) {
        if let Some(peer_info) = self.peers.get_mut(peer_id) {
            peer_info.reputation += delta;
            if peer_info.reputation > self.unban_threshold && self.blacklist.contains(peer_id) {
                self.unban_peer(peer_id);
            }
        }
    }
    
    pub fn decrease_reputation(&mut self, peer_id: &PeerId, delta: i32) {
        if let Some(peer_info) = self.peers.get_mut(peer_id) {
            peer_info.reputation -= delta;
            peer_info.misbehavior_count += 1;
            if peer_info.reputation < self.ban_threshold {
                self.ban_peer(peer_id);
            }
        }
    }
    
    pub fn is_banned(&self, peer_id: &PeerId) -> bool {
        self.blacklist.contains(peer_id)
    }
    
    pub fn ban_peer(&mut self, peer_id: &PeerId) {
        self.blacklist.insert(*peer_id);
        if let Some(peer_info) = self.peers.get_mut(peer_id) {
            peer_info.reputation = -200;
        }
        tracing::warn!("Banned peer {}", peer_id);
    }
    
    pub fn unban_peer(&mut self, peer_id: &PeerId) {
        self.blacklist.remove(peer_id);
        if let Some(peer_info) = self.peers.get_mut(peer_id) {
            peer_info.reputation = 0;
        }
        tracing::info!("Unbanned peer {}", peer_id);
    }
    
    pub fn get_peer(&self, peer_id: &PeerId) -> Option<&PeerInfo> {
        self.peers.get(peer_id)
    }
    
    pub fn is_validator(&self, peer_id: &PeerId) -> bool {
        self.validators.contains(peer_id)
    }
    
    pub fn validators(&self) -> &HashSet<PeerId> {
        &self.validators
    }
}
